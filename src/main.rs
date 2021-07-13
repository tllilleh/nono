use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::process;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct Record {
    sizeCol: usize,
    sizeRow: usize,
    title: String,
    number: i32,
    solution: String,
    difficulty: String,
    colClues: String,
    rowClues: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Puzzle {
    title: String,
    number: i32,
    solution: String,
    difficulty: String,
    rows: Vec<Vec<usize>>,
    cols: Vec<Vec<usize>>,
}

const BOX_EMPTY: char = '█';
const BOX_FILLED: char = ' ';
const BOX_UNKNOWN: char = '?';

fn solve_puzzle(rows: Vec<Vec<usize>>, cols: Vec<Vec<usize>>) {
    // println!("rows {:?}", rows);
    // println!("cols {:?}", cols);

    // Initialize rows
    println!("initializing rows");
    let mut row_combos: Vec<Vec<Vec<char>>> = rows
        .par_iter()
        .map(|row| create_combos(&row, cols.len()))
        .collect();
    println!("initializing rows: done");

    // Initialize cols
    println!("initializing cols");
    let mut col_combos: Vec<Vec<Vec<char>>> = cols
        .par_iter()
        .map(|col| create_combos(&col, rows.len()))
        .collect();
    println!("initializing cols: done");

    let mut done_solving = false;
    let mut step = 0;
    while !done_solving {
        done_solving = true;
        step += 1;
        println!("step: {}", step);

        // Create masks
        let mut row_masks = Vec::<Vec<char>>::new();
        let mut col_masks = Vec::<Vec<char>>::new();

        rayon::join(
            || {
                row_masks = row_combos
                    .par_iter()
                    .map(|combos| create_mask(&combos))
                    .collect();
            },
            || {
                col_masks = col_combos
                    .par_iter()
                    .map(|combos| create_mask(&combos))
                    .collect();
            },
        );

        row_masks
            .iter_mut()
            .enumerate()
            .for_each(|(row, row_mask)| {
                col_masks
                    .iter_mut()
                    .enumerate()
                    .for_each(|(col, col_mask)| {
                        if row_mask[col] == BOX_UNKNOWN {
                            row_mask[col] = col_mask[row];
                        } else if col_mask[row] == BOX_UNKNOWN {
                            col_mask[row] = row_mask[col];
                        } else if row_mask[col] != col_mask[row] {
                            process::exit(1);
                        }
                    });
            });

        rayon::join(
            || {
                // Update rows
                row_combos = row_combos
                    .par_iter()
                    .zip(&row_masks)
                    .map(|(combos, masks)| filter_with_mask(&combos, &masks))
                    .collect();
            },
            || {
                // Update cols
                col_combos = col_combos
                    .par_iter()
                    .zip(&col_masks)
                    .map(|(combos, masks)| filter_with_mask(&combos, &masks))
                    .collect();
            },
        );

        // Show progress
        for row_mask in &row_masks {
            let mut line = "".to_string();
            for ch in row_mask {
                if *ch == BOX_UNKNOWN {
                    done_solving = false;
                }
                line.push(*ch);
                line.push(*ch);
            }
            println!("{}", line);
        }
        println!();
    }

    println!("Solved in {} steps.", step);
}

fn create_mask(combos: &[Vec<char>]) -> Vec<char> {
    let mut mask = Vec::<char>::new();

    if let Some((first_combo, remaining_combos)) = combos.split_first() {
        let length = first_combo.len();
        mask = first_combo.to_vec();

        for combo in remaining_combos {
            for ii in 0..length {
                if mask[ii] != combo[ii] {
                    mask[ii] = BOX_UNKNOWN;
                }
            }
        }
    }

    mask
}

fn filter_with_mask(combos: &[Vec<char>], mask: &[char]) -> Vec<Vec<char>> {
    combos
        .iter()
        .filter(|combo| {
            for (m, c) in mask.iter().zip(combo.iter()) {
                if *m != BOX_UNKNOWN && c != m {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

fn create_combos(chunk_list: &[usize], total_size: usize) -> Vec<Vec<char>> {
    let trailing_spaces;
    let mut combos = Vec::<Vec<char>>::new();

    if chunk_list.is_empty() {
        return combos;
    } else if chunk_list.len() > 1 {
        trailing_spaces = 1;
    } else {
        trailing_spaces = 0;
    }

    let min_size = min_size_of_chunk_list(&chunk_list);

    if min_size > total_size {
        return combos;
    }

    if let Some((first_chunk, remaining_chunk_list)) = chunk_list.split_first() {
        // println!("first_chunk: {:?}", first_chunk);
        // println!("remaining_chunk_list: {:?}", remaining_chunk_list);

        let remaining_min_size = min_size_of_chunk_list(remaining_chunk_list);

        for leading_space in
            0..((total_size - remaining_min_size) - (first_chunk + trailing_spaces) + 1)
        {
            let mut line;

            line = vec![BOX_EMPTY; leading_space];
            line.extend(&vec![BOX_FILLED; *first_chunk]);
            line.extend(&vec![BOX_EMPTY; trailing_spaces]);
            // println!("line: {:?}", line);

            let tail_combos = create_combos(remaining_chunk_list, total_size - line.len());

            if tail_combos.is_empty() {
                line.extend(&vec![BOX_EMPTY; total_size - line.len()]);
                combos.push(line);
            } else {
                for tail in tail_combos {
                    let mut new_line = Vec::<char>::new();

                    for ch in &line {
                        new_line.push(*ch);
                    }
                    for ch in &tail {
                        new_line.push(*ch);
                    }
                    combos.push(new_line);
                }
            }
        }
    }

    combos
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_create_combos() {
        let chunk_list = vec![1, 2, 3];
        assert_eq!(create_combos(chunk_list, 10), Vec::<usize>::new());
    }
}

fn min_size_of_chunk_list(chunk_list: &[usize]) -> usize {
    if chunk_list.is_empty() {
        return 0;
    }

    let mut size: usize = 0;
    for chunk in chunk_list {
        size += chunk;
    }

    size = size + chunk_list.len() - 1;
    size
}

fn main() {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut cols: Vec<Vec<usize>> = Vec::new();

    // http://www.nonograms.org/nonograms/i/xxx

    // panda on tree
    //if let Ok(puzzle) = load_from_json(18264) {

    // heron
    //if let Ok(puzzle) = load_from_json(3541) {

    // dinosaur
    //if let Ok(puzzle) = load_from_json(4091) {

    // donkey
    if let Ok(puzzle) = load_from_json(18274) {

        solve_puzzle(puzzle.rows, puzzle.cols);
    }

    //convert_csv_to_json();
}

fn load_from_json(number : i32) -> Result<Puzzle, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open("puzzles.json")?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let puzzles : Vec<Puzzle> = serde_json::from_reader(reader)?;

    // Return the `User`.
    for puzzle in puzzles {
        if puzzle.number == number {
            return Ok(puzzle.clone());
        }
    }

    Err("not found".into())
}

fn convert_csv_to_json() -> Result<(), Box<dyn Error>> {
    let mut puzzles: Vec<Puzzle> = Vec::new();

    let mut rdr = csv::Reader::from_reader(io::stdin());
    let results = rdr.deserialize();

    for result in results {
        let record: Record = result?;
        //println!("{:?}", record);
        //println!("{}: {},{}", record.title, record.sizeRow, record.sizeCol);
        //println!("  size: {},{}", record.sizeRow, record.sizeCol);
        let mut cols: Vec<Vec<usize>> = Vec::new();
        let mut rows: Vec<Vec<usize>> = Vec::new();

        let rowClues: Vec<usize> = record
            .rowClues
            .split(",")
            .map(|i| match i.parse() {
                Ok(num) => num,
                Err(_) => 0,
            })
            .collect();
        //println!("  row clues: {:?}", rowClues);
        let mut ii = 0;
        for _ in 0..record.sizeRow {
            cols.push(Vec::new());
        }
        for num in rowClues {
            if num != 0 {
                cols[ii].push(num);
            }
            ii = (ii + 1) % record.sizeRow;
        }
        //println!("  cols: {:?}", cols);

        let colClues: Vec<usize> = record
            .colClues
            .split(",")
            .map(|i| match i.parse() {
                Ok(num) => num,
                Err(_) => 0,
            })
            .collect();
        //println!("  col clues: {:?}", colClues);
        let clues_per_col = colClues.len() / record.sizeCol;
        ii = 0;
        for col in colClues.chunks(clues_per_col) {
            rows.push(Vec::new());
            for num in col {
                if *num != 0 {
                    rows[ii].push(*num);
                }
            }
            ii += 1;
        }
        //println!("  rows: {:?}", rows);

        puzzles.push(Puzzle {
            title: record.title,
            number: record.number,
            solution: record.solution,
            difficulty: record.difficulty,
            cols,
            rows,
        });
    }

    println!("{}", serde_json::to_string(&puzzles).unwrap());

    Ok(())
}
