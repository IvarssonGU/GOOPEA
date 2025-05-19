use std::path::PathBuf;
fn test_file(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(filename)
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_parse_lex {
    use crate::grammar;
    use crate::lexer::Lexer;
    use std::fs;
    use std::path::Path;

    fn parse_example(path: &Path) -> () {
        let code = fs::read_to_string(path).unwrap();
        println!(
            "{:?}",
            grammar::ProgramParser::new()
                .parse(Lexer::new(&code))
                .unwrap()
        );
    }

    #[test]
    fn parse_reverse() {
        parse_example(Path::new("examples/reverse.goo"));
    }

    #[test]
    fn parse_zipper_tree() {
        parse_example(Path::new("examples/zipper_tree.goo"));
    }

    #[test]
    fn parse_integer() {
        parse_example(Path::new("examples/integer.goo"));
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_preprocessor {
    use super::test_file;
    use crate::preprocessor::preprocess;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_str(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn preprocessor_1() {
        let code = preprocess(test_file("test_1.goo"));
        println!("{code}");
        assert_eq!(hash_str(&code), 6592751422363609175);
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_interpreter {
    use super::test_file;
    use crate::interpreter;
    use interpreter::{_compile, Interpreter};

    #[test]
    fn interpreter_0() {
        let core_ir = _compile(test_file("test_0.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(interpreter.get_return_value().unwrap().unwrap_val(), 8);
    }

    #[test]
    fn interpreter_1() {
        let core_ir = _compile(test_file("test_1.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(
            interpreter.get_return_value().unwrap().unwrap_val(),
            12345789
        );
    }

    #[test]
    fn interpreter_2() {
        let core_ir = _compile(test_file("test_2.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(
            interpreter.get_return_format(),
            "[2, 3, 4, 6, 5]"
        );
    }

    #[test]
    fn interpreter_3() {
        let core_ir = _compile(test_file("test_3.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(
            interpreter.get_return_format(),
            "([B: [B: 0, 3], 4], [B: 6], [B: 5])"
        );
    }

    #[test]
    fn interpreter_4() {
        let core_ir = _compile(test_file("test_4.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(
            interpreter.get_return_format(),
            "[A: 0, [A: 1, [A: 2, [A: 3, [A: [E: 5], 1]]]]]"
        );
    }

    #[test]
    fn interpreter_5() {
        let core_ir = _compile(test_file("test_5.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(
            interpreter.get_return_format(),
            "[-2, -1, 0, 1]"
        );
    }

    #[test]
    fn interpreter_6() {
        let core_ir = _compile(test_file("test_6.goo"));
        let mut interpreter = Interpreter::from_program(&core_ir);
        interpreter.run_until_done();
        assert_eq!(interpreter.get_return_format(), "[2, 3]");
    }
}
