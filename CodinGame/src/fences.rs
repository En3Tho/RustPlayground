use std::io;
use std::cmp::max;

fn read_line_as_i32() -> i32 {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    input_line.trim().parse::<i32>().unwrap()
}

fn parse_num(value: &str) -> i32 {
    value.parse::<i32>().unwrap()
}

fn write_answer(st: usize, ed: usize) -> () {
    println!("{} {}", st, ed)
}

struct Block {
    start: usize,
    end: usize
}

fn try_get_next_block(start_index: usize, blocks: &Vec<Block>) -> Option<(Block, usize)> {
    if start_index >= blocks.len() {
        None
    }
    else {
        let mut found = false;
        let mut index = start_index + 1;
        let mut result = Block { start: blocks[start_index].start, end: blocks[start_index].end };
        while !found && index < blocks.len() {
            let next = &blocks[index];
            if result.end >= next.start {
                result = Block { start: result.start, end: max(result.end, next.end) };
                index += 1;
            }
            else {
                found = true;
            }
        }
        Some((result, index))
    }
}

fn go(current_block: &Block, index: usize, length: usize, blocks: &Vec<Block>) -> () {
    match try_get_next_block(index, blocks) {
        Some((block, next_index)) => {
            eprintln!("Found next block: {} {}", block.start, block.end);
            if block.start == 0 && block.end == length {
                println!("All painted");
            } else {
                if block.start != current_block.start {
                    write_answer(current_block.start, block.start);
                }
                go(&Block { start: block.end, end: current_block.end }, next_index, length, blocks);
            }
        },
        None => {
            eprintln!("No blocks found. Current: {} {}", current_block.start, current_block.end);
            if current_block.start != length {
                write_answer(current_block.start, current_block.end);
            }
        }
    }
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

fn read_block() -> Block {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").map(parse_num).collect::<Vec<_>>();
    let st = inputs[0] as usize;
    let ed = inputs[1] as usize;
    Block { start: st, end: ed }
}

fn read_blocks(n: usize) -> Vec<Block> {
    let mut blocks: Vec<Block> = vec! [];

    for _ in 0..n as usize {
        blocks.push(read_block());
    }
    blocks.sort_by(| prev, next | (prev.start, prev.end).partial_cmp(&(next.start, next.end)).unwrap());
    blocks
}

pub fn fences() {
    let l = read_line_as_i32() as usize;
    let n = read_line_as_i32() as usize;
    let blocks = read_blocks(n);
    go(&Block { start: 0, end: l }, 0, l, &blocks);
}