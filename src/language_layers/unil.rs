use crate::language_layer;

language_layer! {
    language = unil;
    extension = "uni";

    process(source, filename) {
        crate::unil::parse(source, filename)
    }
}