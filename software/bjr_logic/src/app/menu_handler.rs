use alloc::format;
use core::fmt::{Display, Formatter, Write};
use core::str::FromStr;
use menu::{Item, ItemType, Menu, Parameter, Runner};
use strum::VariantNames;
use strum_macros::EnumVariantNames;
use bsp_traits::Reader;
use crate::app::event::GlobEvent;
use crate::app::event_queue::get_event_queue;
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::str_to_display;

const READ_BUF_LEN:usize = 64;
#[derive(Clone)]
pub enum MenuError {
    EventBufferOverflow
}
impl Display for MenuError{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            MenuError::EventBufferOverflow => {write!(f,"Menu Event Buffer Overflow")}
        }
    }
}
impl Severity for MenuError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            MenuError::EventBufferOverflow => {ErrorSeverity::Panic}
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

                }
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
    if let Ok(event) = GlobEvent::from_str(args[0]) {
        if let Err(error) = get_event_queue().enqueue(event) {
            context.error = Err(MenuError::EventBufferOverflow);
        }
    }else {
        interface.write_str("Couldn't parse input. Available events are:\n").unwrap();
        for v in GlobEvent::VARIANTS {
            interface.write_str(v).unwrap();
            interface.write_str("\n").unwrap();
        }

    }
}
fn list_events<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    _context: &mut Context,
) {
    for v in GlobEvent::VARIANTS {
        interface.write_str(v).unwrap();
        interface.write_str("\n").unwrap();
    }
}