#[macro_export]
macro_rules! lang_hook {
    {
        process($layers: ident) 
            $body: block
    } => {
        pub fn define($layers: &mut std::collections::HashMap<std::rc::Rc<str>, $crate::LanguageLayerFn>) {
            $body
        }

        #[cfg(feature = "dev")]
        pub fn generate_headers(_globals: &std::collections::HashMap<std::rc::Rc<str>, $crate::compiler::type_system::UniLType>) -> String {
            String::new()
        }
    };
    {
        process($layers: ident) 
            $body: block

        generate_headers($globals: ident)
            $headers: block
    } => {
        pub fn define($layers: &mut std::collections::HashMap<std::rc::Rc<str>, $crate::LanguageLayerFn>) {
            $body
        }

        #[cfg(feature = "dev")]
        pub fn generate_headers($globals: &std::collections::HashMap<std::rc::Rc<str>, $crate::compiler::type_system::UniLType>) -> String {
            $headers
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
        language = $language: ident;
        extension = $extension: literal;

        process($source: ident, $filename: ident)
            $process: block

        $(
            generate_headers($globals: ident)
                $headers: block
        )?
    } => {
        pub mod $language {
            #[allow(dead_code)]
            pub const EXTENSION: &str = $extension;

            pub fn process(
                $source: String,
                $filename: std::rc::Rc<str>
            ) -> Result<Vec<$crate::Expression>, Vec<String>> {
                $process
            }
        }

        $crate::lang_hook! {
            process(layers) {
                $crate::lang_define!(layers, $language::process, $extension);
            }

            $(generate_headers($globals) $headers)?
        }
    };
}

#[cfg(feature = "dev")]
#[macro_export]
macro_rules! __write_header_file {
    ($language: ident, $globals: ident, $folder: literal) => {
        {
            use std::io::Write;
            
            let result = $language::generate_headers($globals);
            if result != "" {
                let mut f = std::fs::File::create(std::path::Path::new($folder).join(format!("UniV.{}", $language::$language::EXTENSION)))?;
                f.write_all(result.as_bytes())?;
            }
        }
    };
}