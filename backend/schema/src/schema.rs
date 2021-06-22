use super::Namespace;

#[derive(Debug)]
pub struct Schema {
    namespaces: Vec<Namespace>,
}

impl Schema {
    pub fn new(namespaces: Vec<Namespace>) -> Self {
        Schema {
            namespaces,
        }
    }

    pub fn empty() -> Self {
        Schema {
            namespaces: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    pub fn it_works() {
        let schema = parse(
            "
            first_namespace {
                - field1: u8
                - field2: string

                first_inner {
                    - field3: i32

                    nested {
                        - field4: string
                    }
                }

                second_inner {
                    - field5: u32
                }
            }

            second_namespace {
                - field6: u8
                - field7: blob

                first_inner {
                    - field8: u8
                }
            }
        ",
        )
        .unwrap();

        dbg!(&schema);

        assert_eq!(6, schema.len());
    }
}
