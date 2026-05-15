use crate::papermc_enum;

papermc_enum! {
    /// Controls what happens after the player interacts with the dialog.
    pub DialogAfterAction in "io/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction" {
        Close => "CLOSE",
        None => "NONE",
        WaitForResponse => "WAIT_FOR_RESPONSE",
    }
}
