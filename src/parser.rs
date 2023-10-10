use log::{debug, error};
use rocket::tokio;
use std::{
    future::Future,
    io::{Error, ErrorKind},
    pin::Pin,
};

use crate::{types::Message, WebexClient};

// ###########################################################################
// Tuple definition that contains the name:value mapping.
// ###########################################################################

pub type ArgTuple = Vec<(std::string::String, std::string::String)>;
pub type Callback = Box<
    dyn Fn(
            &WebexClient,
            Message,
            &ArgTuple,
            &ArgTuple,
        ) -> (Pin<Box<dyn Future<Output = ()> + Send>>)
        + Send
        + Sync,
>;

// ###################################################################
// Define the Argument trait
// ###################################################################

pub trait Argument: Send + Sync {
    fn name(&self) -> &str;
    fn is_required(&self) -> bool;
}

// ###################################################################
// Define the RequiredArgument struct implementing the Argument trait.
// ###################################################################

pub struct RequiredArgument<T: Send + Sync> {
    name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync> RequiredArgument<T> {
    pub fn new(name: &str) -> Self {
        RequiredArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync> Argument for RequiredArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        true
    }
}

// ###################################################################
// Define the OptionalArgument struct implementing the Argument trait.
// ###################################################################

pub struct OptionalArgument<T> {
    name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OptionalArgument<T> {
    pub fn new(name: &str) -> Self {
        OptionalArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync> Argument for OptionalArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        false
    }
}

// ###################################################################
// Structure for a final parsed command.
// ###################################################################

pub struct Command<'a> {
    pub command: String,
    pub required_arguments: ArgTuple,
    pub optional_arguments: ArgTuple,
    pub callback: &'a Callback,
}

impl<'a> Command<'a> {
    const INVALID_CMD: &str = "You have entered an invalid Command!";
    const NO_CMD: &str = "Command was not specified!";
    // const INVALID_SYNTAX: &str = "Sorry, I could not understand you.";
    const MISSING_ARGS: &str = "Missing required variables.";

    pub fn invalid(error: &str) -> Error {
        Error::new(ErrorKind::InvalidData, format!("{}", error))
    }

    pub async fn callback<F, Fut>(
        &self,
        client: &WebexClient,
        message: Message,
        required_argument: &ArgTuple,
        optional_arguments: &ArgTuple,
        f: F,
    ) where
        F: Fn(&WebexClient, Message, &ArgTuple, &ArgTuple) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(f(client, message, required_argument, optional_arguments));
    }
}

// ###################################################################
// Define the Parser struct
// ###################################################################

pub(crate) struct Parser {
    commands: std::collections::HashMap<String, (Callback, Vec<Box<dyn Argument>>)>,
}

impl<'a> Parser {
    pub fn new() -> Self {
        Parser {
            commands: std::collections::HashMap::new(),
        }
    }

    // ------------------------------------------------------------------------------
    // Append a command to the available (parsable). list of commands
    // ------------------------------------------------------------------------------

    /**
     * Arguments to be stored:
     *
     * Client: The webex client designated for listenining the incoming request.
     * Command: The command string we want to listen for.
     * Args: The vector of required/optional args that conform that specific command.
     * Callback: The custom user defined function that contains the command implementation.
     */

    pub fn add_command(&mut self, command: &str, args: Vec<Box<dyn Argument>>, callback: Callback) {
        self.commands.insert(command.to_string(), (callback, args));
    }

    // ------------------------------------------------------------------------------
    // Retrieve the set of arguments required for proper command execution.
    // ------------------------------------------------------------------------------

    pub fn get_command_arguments(&self, name: &str) -> &Vec<Box<dyn Argument>> {
        &self.commands.get(name).unwrap().1
    }

    // ------------------------------------------------------------------------------
    // Parse the plain text string values into a usable command.
    // ------------------------------------------------------------------------------

    pub fn parse(&self, plain_string_message: String) -> Result<Command, Error> {
        // Separate the bot name from the actual command and arguments. \/?\w+
        let parts = plain_string_message.split(' ').collect::<Vec<&str>>();
        let num_parts = parts.len();
        if num_parts <= 1 {
            error!("No command has been specified!");
            Err::<Command, Error>(Command::invalid(Command::NO_CMD));
        }

        // If the command is correctly initialized, check if it is available as
        // a key within the hasmap.
        if !self.commands.contains_key(parts[1]) {
            error!("Command not found!");
            Err::<Command, Error>(Command::invalid(Command::INVALID_CMD));
        }

        // If the commands is present within the registered commands, retrive the
        // structure information.
        let command_structure = self.commands.get(parts[1]).unwrap();
        let arguments_len = command_structure.1.len();

        let mut required_arguments = Vec::<(String, String)>::new();
        let mut optional_arguments = Vec::<(String, String)>::new();

        // Check if the required arguments list is satisfied.
        if arguments_len >= num_parts - 2 {
            for index in 0..num_parts - 2 {
                if command_structure.1[index].is_required() {
                    debug!("Required command: {}", command_structure.1[index].name());
                    required_arguments.push((
                        command_structure.1[index].name().to_string(),
                        parts[index].to_string(),
                    ));
                } else {
                    debug!("Optional command: {}", command_structure.1[index].name());
                    optional_arguments.push((
                        command_structure.1[index].name().to_string(),
                        parts[index].to_string(),
                    ));
                }
            }
        } else {
            error!("Did not specified all the required arguments to execute this command!");
            Err::<Command, Error>(Command::invalid(Command::MISSING_ARGS));
        }

        debug!("Calling the callback function supplied on the argument list.");
        // Execute the callback function.
        // command_structure.0(&required_arguments, &optional_arguments);

        // Return the final parsed command with its respective required/optional
        // arguments classified.

        let command = Command {
            command: parts[1].to_string(),
            optional_arguments: optional_arguments,
            required_arguments: required_arguments,
            callback: &command_structure.0,
        };

        Ok(command)
    }
}
