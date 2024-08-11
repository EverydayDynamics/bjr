use core::fmt::Write;
use menu::{Item, ItemType, Menu, Parameter, Runner};
use bsp_traits::Reader;
const READ_BUF_LEN:usize = 64;

pub struct MenuContext{
   pub data: Context,
    pub buffer: [u8;READ_BUF_LEN],
}
impl Default for MenuContext {
    fn default() -> Self {
        MenuContext{ data: Default::default(), buffer: [0u8;READ_BUF_LEN] }
    }
}
#[derive(Default)]
struct Context {
    _inner: u32,
}
pub struct MenuHandler<'a, MIO: Reader+Write> {
    context: &'a mut Context,
    runner: Runner<'a, MIO, Context>
}
impl<MIO> MenuHandler<'_, MIO>
where
    MIO: Reader+Write
{
    pub fn new<'a>(menu_io: MIO, context: &'a mut MenuContext) ->MenuHandler<'a, MIO> {
        let root_menu =  Menu {
            label: "ROOT",
            items: &[
                &Item{
                    command: "asdfddd",
                    help: Some("yes"),
                    item_type: ItemType::Callback { function: select_baz, parameters: &[Parameter::Mandatory { parameter_name: "length", help: Some("lenght of the girth") }] },
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
    pub fn update(&mut self) {
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
    }

}
fn select_baz<MIO: Reader+Write>(
    _menu: &Menu<MIO, Context>,
    _item: &Item<MIO, Context>,
    args: &[&str],
    interface: &mut MIO,
    _context: &mut Context,
) {
    writeln!(interface, "In select_baz: Args = {:?}", args).unwrap();
}