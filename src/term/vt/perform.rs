use super::Params;

pub trait Perform {
    fn print(&mut self, c: char);

    fn print_ascii(&mut self, text: &[u8]) {
        for &byte in text {
            self.print(char::from(byte));
        }
    }

    fn execute(&mut self, byte: u8);

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], action: u8);

    fn esc_dispatch(&mut self, intermediates: &[u8], byte: u8);

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool);

    fn apc_dispatch(&mut self, _data: &[u8]) {}
}
