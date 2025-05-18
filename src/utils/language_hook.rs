#[macro_export]
macro_rules! lang_hook {
    {
        $layers: ident $body: expr
    } => {
        pub fn define($layers: &mut std::collections::HashMap<std::rc::Rc<str>, $crate::LanguageLayerFn>) {
            $body
        }
    };
}

#[macro_export]
macro_rules! lang_define {
    ($layers: ident, $function: expr, $extension: literal) => {
        $layers.insert(std::rc::Rc::from($extension), $function);
    };
}

#[macro_export]
macro_rules! language_layer {
    {
        extension = $extension: literal;

        $language: ident :: process($source: ident, $filename: ident)
            $process: expr
    } => {
        mod $language {
            pub fn process(
                $source: String,
                $filename: std::rc::Rc<str>
            ) -> Result<Vec<$crate::Expression>, Vec<String>> {
                $process
            }
        }

        $crate::lang_hook! {
            layers {
                $crate::lang_define!(layers, $language::process, $extension);
            }
        }
    };
}