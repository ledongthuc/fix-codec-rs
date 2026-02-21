use crate::error::FixError;
use crate::message::Message;

pub fn encode(_msg: &Message<'_>, _out: &mut Vec<u8>) -> Result<(), FixError> {
    todo!()
}
