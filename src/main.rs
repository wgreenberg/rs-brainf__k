use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::Index;
use std::ops::IndexMut;

// > go right on strip
// < left
// + increment current cell
// - decr
// [ start loop
// ] if curr cell != 0, jmp to [
// . print at current (ascii)
// , read at current

#[derive(Debug)]
#[derive(PartialEq)]
enum BfError {
    MismatchedBraces,
    Segfault,
}

type BfStateResult = Result<(), BfError>;

#[derive(Debug)]
struct GrowableVec {
    arr: Vec<u8>,
    default_value: u8,
}

impl Index<usize> for GrowableVec {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        if index >= self.arr.len() {
            return &self.default_value;
        }
        return &self.arr[index];
    }
}

impl IndexMut<usize> for GrowableVec {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut u8 {
        if index >= self.arr.len() {
            self.arr.resize(index + 1, self.default_value);
        }
        return &mut self.arr[index];
    }
}

impl GrowableVec {
    fn new() -> GrowableVec {
        let arr: Vec<u8> = Vec::new();
        GrowableVec {
            arr: arr,
            default_value: 0,
        }
    }
}

struct BfState {
    memory: GrowableVec,
    pointer: usize,
}

impl BfState {
    fn new() -> BfState {
        BfState {
            memory: GrowableVec::new(),
            pointer: 0,
        }
    }

    fn curr(&self) -> u8 {
        return self.memory[self.pointer];
    }

    /*
    // XXX: Why don't we need a lifetime annotation here???
    fn curr_mut(&mut self) -> &mut u8 {
        return &mut self.memory[self.pointer];
    }
    // XXX corresponding test to bizarre behavior above
    #[test]
    fn test_curr_mut() {
        let mut state = BfState::new();

        let mut num = state.curr_mut();
        *num = 10;
        assert_eq!(state.curr(), 10);
    }
    */

    fn set_curr(&mut self, value: u8) {
        self.memory[self.pointer] = value;
    }

    fn inc(&mut self) {
        let (result, _) = self.curr().overflowing_add(1);
        self.set_curr(result);
    }

    fn dec(&mut self) {
        let (result, _) = self.curr().overflowing_sub(1);
        self.set_curr(result);
    }

    fn left(&mut self) -> BfStateResult {
        if self.pointer == 0 {
            return Err(BfError::Segfault);
        }
        self.pointer -= 1;
        Ok(())
    }

    fn right(&mut self) {
        self.pointer += 1;
    }
}

fn read() -> u8 {
    return io::stdin().bytes().next().expect("reached end of stdin").expect("failed to extract bytes");
}

fn write(c: u8) {
    print!("{}", c as char);
    io::stdout().flush().expect("stdout.flush() failed");
}

// Constructs a mapping of the program counter (pc) to jump to at any given
// square bracket in the program
fn build_pc_map(program: &str) -> Result<Vec<usize>, BfError> {
    let mut stack: Vec<usize> = Vec::new();
    let mut map: Vec<usize> = Vec::new();
    map.resize(program.len(), 0);

    for (index, sym) in program.char_indices() {
        if sym == '[' {
            stack.push(index);
        }
        if sym == ']' {
            match stack.pop() {
                None => return Err(BfError::MismatchedBraces),
                Some(left_pc) => {
                    map[index] = left_pc;
                    map[left_pc] = index;
                },
            };
        }
    }
    if !stack.is_empty() {
        return Err(BfError::MismatchedBraces);
    }

    return Ok(map);
}

fn run(program: &str, state: &mut BfState) -> BfStateResult {
    let pc_map = match build_pc_map(program) {
        Err(e) => return Err(e),
        Ok(map) => map,
    };

    let mut pc = 0;
    let symbols: Vec<char> = program.chars().collect();
    while pc < symbols.len() {
        let sym = symbols[pc];
        let result = match sym {
            '+' => Ok(state.inc()),
            '-' => Ok(state.dec()),
            '>' => Ok(state.right()),
            '<' => state.left(),
            ',' => Ok(state.set_curr(read())),
            '.' => Ok(write(state.curr())),
            '[' => {
                if state.curr() == 0 {
                    pc = pc_map[pc];
                }
                Ok(())
            },
            ']' => {
                pc = pc_map[pc] - 1;
                Ok(())
            },
            _ => Ok(()),
        };
        if result.is_err() {
            return result;
        }
        pc = pc + 1;
    }
    return Ok(());
}

fn main() {
    for arg in env::args().skip(1) {
        let mut buf = String::new();
        let mut file = File::open(arg).expect("couldn't open that file bro");
        file.read_to_string(&mut buf).expect("couldn't read from file");
        let mut state = BfState::new();
        run(buf.trim(), &mut state).expect("error running bf program!");
        println!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GrowableVec
    #[test]
    fn can_index_growable_vects() {
        let mut vect = GrowableVec::new();
        vect[0] = 33;
        assert_eq!(vect[0], 33);
    }

    #[test]
    fn growable_vects_have_sensible_index_defaults() {
        let vect = GrowableVec::new();
        assert_eq!(vect[0], 0);
        assert_eq!(vect[33], 0);
    }

    // BfState
    #[test]
    fn test_inc() {
        let mut state = BfState::new();
        state.inc();
        assert_eq!(state.curr(), 1);
        state.inc();
        assert_eq!(state.curr(), 2);
    }

    #[test]
    fn test_dec() {
        let mut state = BfState::new();
        state.memory[state.pointer] = 200;
        state.dec();
        assert_eq!(state.curr(), 199);
        state.dec();
        assert_eq!(state.curr(), 198);
    }

    #[test]
    fn test_curr() {
        let mut state = BfState::new();
        assert_eq!(state.curr(), 0);

        state.pointer = 13;
        assert_eq!(state.curr(), 0);
        state.memory[state.pointer] = 40;
        assert_eq!(state.curr(), 40);
    }


    #[test]
    fn increment_overflow_test() {
        let mut state = BfState::new();
        state.memory[0] = 255;
        state.inc();
        assert_eq!(state.curr(), 0);
    }

    #[test]
    fn decrement_underflow_test() {
        let mut state = BfState::new();
        state.memory[0] = 0;
        state.dec();
        assert_eq!(state.curr(), 255);
    }

    #[test]
    fn set_curr_test() {
        let mut state = BfState::new();
        state.set_curr(10);
        assert_eq!(state.curr(), 10);
    }

    #[test]
    fn right() {
        let mut state = BfState::new();
        assert_eq!(state.pointer, 0);
        state.right();
        assert_eq!(state.pointer, 1);
        state.right();
        assert_eq!(state.pointer, 2);
    }

    #[test]
    fn run_left() {
        let mut state = BfState::new();
        state.pointer = 200;
        assert!(state.left().is_ok());
        assert_eq!(state.pointer, 199);
        assert!(state.left().is_ok());
        assert_eq!(state.pointer, 198);
    }

    #[test]
    fn run_left_handle_segfault() {
        let mut state = BfState::new();
        let result = run("<", &mut state);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(BfError::Segfault));
    }

    #[test]
    fn run_propagates_segfault_err() {
        let result = run("<", &mut BfState::new());
        assert!(result.is_err());
        assert_eq!(result.err(), Some(BfError::Segfault));
    }

    #[test]
    fn run_ok_on_empty_program() {
        assert!(run("", &mut BfState::new()).is_ok());
    }

    #[test]
    fn run_empty_loop() {
        let mut state = BfState::new();
        assert!(run("[]", &mut state).is_ok());
    }

    #[test]
    fn run_nonempty_loop() {
        let mut state = BfState::new();
        assert!(run("++[>+<-]", &mut state).is_ok());
        assert_eq!(state.memory[0], 0);
        assert_eq!(state.memory[1], 2);
    }

    #[test]
    fn run_loop_with_overflow() {
        let mut state = BfState::new();
        assert!(run("-[->+<]", &mut state).is_ok());
        assert_eq!(state.memory[0], 0);
        assert_eq!(state.memory[1], 255);

        state = BfState::new();
        assert!(run("[+]", &mut state).is_ok());
        assert!(run("+[+>+<]", &mut state).is_ok());
        assert_eq!(state.memory[0], 0);
        assert_eq!(state.memory[1], 255);
    }

    #[test]
    fn run_noop_loop() {
        assert!(run("[<]", &mut BfState::new()).is_ok());
    }

    #[test]
    fn run_nested_loops() {
        let mut state = BfState::new();
        assert!(run("-[->+<]", &mut state).is_ok());
        assert_eq!(state.memory[0], 0);
        assert_eq!(state.memory[1], 255);
    }

    #[test]
    fn run_fails_on_mismatched_parens() {
        let mut state = BfState::new();
        let mut result = run("[]]", &mut state);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(BfError::MismatchedBraces));

        result = run("[[]", &mut state);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(BfError::MismatchedBraces));

        result = run("]", &mut state);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(BfError::MismatchedBraces));
    }

    #[test]
    fn run_nontrivial_empty_loops() {
        assert!(run("[[[]]]", &mut BfState::new()).is_ok());
        assert!(run("[][][]", &mut BfState::new()).is_ok());
    }
}
