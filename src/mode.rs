#[derive(Debug)]
pub(crate) enum Mode {
    MostRecentVersion,
    ChooseVersionInteractively,
}

impl Mode {
    pub(crate) fn get_from_args(args: &mut Vec<String>) -> Self {
        if args.last().expect("there's at least one element") == "-V" {
            args.pop().expect("there's at least one element");
            Self::ChooseVersionInteractively
        } else {
            Self::MostRecentVersion
        }
    }
}
