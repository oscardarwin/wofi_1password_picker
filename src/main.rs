use anyhow::Result;
use item::select_field_to_copy;
use item_description::select_item_description;
use session::get_op_session_from_systemd;

mod item;
mod item_description;
mod session;
mod wl_copy;
mod wofi_message;
mod wofi_select;

fn main() -> Result<()> {
    let session = get_op_session_from_systemd()?;
    let item_description = select_item_description(&session)?;
    let field = select_field_to_copy(&session, &item_description)?;
    wl_copy::to_clipboard(field)
}
