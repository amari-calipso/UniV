use crate::language_layer;

language_layer! {
    extension = "uni";

    unil::process(source, filename) {
        crate::unil::parse(source, filename)
    }
}