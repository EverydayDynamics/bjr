use alloc::format;
use core::fmt::{Display, Formatter, Write};
use core::str::FromStr;
use menu::{Item, ItemType, Menu, Parameter, Runner};
use strum::{IntoEnumIterator, VariantNames};
use strum_macros::EnumVariantNames;
use bsp_traits::Reader;
use crate::app::event::GlobEvent;
use crate::app::event_queue::get_event_queue;
use crate::app::menu_handler::MenuError::MenuInterfaceWriteError;
use crate::app::parameter_manager::ParameterList;
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::str_to_display;

const READ_BUF_LEN:usize = 64;
#[derive(Clone)]
pub enum MenuError {
    EventBufferOverflow,
    MenuInterfaceWriteError,
}
impl Display for MenuError{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MenuError::EventBufferOverflow => {write!(f,"Menu Event Buffer Overflow")},
            MenuError::MenuInterfaceWriteError => {write!(f,"Menu Interface Writing Error")},
        }
    }
}
impl Severity for MenuError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            MenuError::EventBufferOverflow => {ErrorSeverity::Panic},
            MenuError::MenuInterfaceWriteError => {ErrorSeverity::Report},
        }
    }
}
pub struct MenuContext{
   pub data: Context,
    pub buffer: [u8;READ_BUF_LEN],
}
impl Default for MenuContext {
    fn default() -> Self {
        MenuContext{ data: Default::default(), buffer: [0u8;READ_BUF_LEN] }
    }
}
struct Context {
    _inner: u32,
    error: Result<(),MenuError>,
}
impl Default for Context {
    fn default() -> Self {
        Context{ _inner: 0,error: Ok(()) }
    }
}
pub struct MenuHandler<'a, MIO: Reader+Write> {
    context: &'a mut Context,
    runner: Runner<'a, MIO, Context>
}
impl<MIO> MenuHandler<'_, MIO>
where
    MIO: Reader+Write
{
    pub fn new(menu_io: MIO, context: &mut MenuContext) ->MenuHandler<MIO> {
        let root_menu =  Menu {
            label: "ROOT",
            items: &[
                &Item{
                    command: "se",
                    help: Some("Sends events directly"),
                    item_type: ItemType::Callback { function: send_event, parameters: &[Parameter::Mandatory { parameter_name: "name", help: Some("name of the event to be sent") }] },

                },
                    &Item{
                    command: "le",
                    help: Some("lists available events"),
                    item_type: ItemType::Callback { function: list_events, parameters: &[] },

                },
                &Item{
                    command: "gp",
                    help: Some("Reads the value of a parameter"),
                    item_type: ItemType::Callback { function: get_parameter, parameters: &[Parameter::Mandatory { parameter_name: "name", help: Some("name of the parameter to be read") }] },
                },
                &Item{
                    command: "lp",
                    help: Some("Lists all the available parameters"),
                    item_type: ItemType::Callback { function: list_parameters, parameters: &[] },
                },
                &Item{
                    command: "sp",
                    help: Some("Sets the value of a parameter"),
                    item_type: ItemType::Callback { function: set_parameter, parameters: &[Parameter::Mandatory { parameter_name: "name", help: Some("name of the parameter to be set") }, Parameter::Mandatory { parameter_name: "value", help: Some("The value to set the parameter to") } ] },
                },
            ],
            entry: None,
            exit: None,
        };
        MenuHandler{
            runner: Runner::new(root_menu, &mut context.buffer, menu_io, &mut context.data),
            context: &mut context.data,
        }
    }
    pub fn update(&mut self)->Result<(),MenuError> {
        self.context.error = Ok(());
        let mut buf = [0u8;READ_BUF_LEN];
        let read_chars = self.runner.interface.read(&mut buf);
        for byte in &mut buf[0..read_chars] {
            let byte_to_send = if *byte as char == '\n' {
                b'\r'
            } else {
               *byte
            };
            self.runner.input_byte(byte_to_send, self.context);

        }
        self.context.error.clone()?;
        Ok(())
    }

}
fn send_event<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    context: &mut Context,
) {
    context.error = fallible_send_event(args, interface);
}
fn fallible_send_event<MIO: Reader+Write>(
    args: &[&str],
    interface: &mut MIO,
) -> Result<(),MenuError> {
    if let Ok(event) = GlobEvent::from_str(args[0]) {
        get_event_queue().enqueue(event).map_err(|_|MenuError::EventBufferOverflow)?;
    }else {
        interface.write_str("Couldn't parse input. Available events are:\n").map_err(|_|MenuInterfaceWriteError)?;
        for v in GlobEvent::VARIANTS {
            interface.write_str(v).map_err(|_|MenuInterfaceWriteError)?;
            interface.write_str("\n").map_err(|_|MenuInterfaceWriteError)?;
        }

    }
    Ok(())
}
fn list_events<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    context: &mut Context,
) {
    context.error = fallible_list_events(interface);
}
fn fallible_list_events<MIO: Reader+Write>(
    interface: &mut MIO,
) -> Result<(),MenuError> {
    for v in GlobEvent::VARIANTS {
        interface.write_str(v).map_err(|_|MenuInterfaceWriteError)?;
        interface.write_str("\n").map_err(|_|MenuInterfaceWriteError)?;
    }
    Ok(())
}
fn get_parameter<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    context: &mut Context,
) {
    context.error = fallible_get_parameter(args, interface);
}
fn fallible_get_parameter<MIO: Reader+Write>(
    args: &[&str],
    interface: &mut MIO,
) -> Result<(),MenuError> {
    if let Ok(parameter) = ParameterList::from_str(args[0]) {
        parameter.write_value(interface).map_err(|_|MenuInterfaceWriteError)?;
    }else {
        interface.write_str("Couldn't parse input.\n").map_err(|_|MenuInterfaceWriteError)?;

    }
    Ok(())
}
fn set_parameter<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    context: &mut Context,
) {
    context.error = fallible_set_parameter(args, interface);
}
fn fallible_set_parameter<MIO: Reader+Write>(
    args: &[&str],
    interface: &mut MIO,
) -> Result<(),MenuError> {
    if let Ok(parameter) = ParameterList::from_str(args[0]) {
        if parameter.set_value(args[1]).is_ok() {
            write!(interface,"{} set to ", parameter).map_err(|_|MenuInterfaceWriteError)?;
        } else {
            write!(interface,"Couldn't parse value for {}", parameter).map_err(|_|MenuInterfaceWriteError)?;
        }
        parameter.write_value(interface).map_err(|_|MenuInterfaceWriteError)?;

    }else {
        interface.write_str("Couldn't parse input.\n").map_err(|_|MenuInterfaceWriteError)?;

    }
    Ok(())
}
fn list_parameters<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    context: &mut Context,
) {
    context.error = fallible_list_parameters(args,interface);
}
fn fallible_list_parameters<MIO: Reader+Write>(
    args: &[&str],
    interface: &mut MIO,
) -> Result<(),MenuError> {
    for param in ParameterList::iter() {
        write!(interface,"{}: ", param).map_err(|_|MenuInterfaceWriteError)?;
        param.write_value(interface).map_err(|_|MenuInterfaceWriteError)?;
        interface.write_str("\n").map_err(|_|MenuInterfaceWriteError)?;
    }
    Ok(())
}
