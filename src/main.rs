use rayon::prelude::*;
use std::process;

const BOX_EMPTY: char = ' ';
const BOX_FILLED: char = 'O';
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

        // create masks
        let mut row_masks: Vec<Vec<char>> = row_combos
            .par_iter()
            .map(|combos| create_mask(&combos))
            .collect();

        let mut col_masks: Vec<Vec<char>> = col_combos
            .par_iter()
            .map(|combos| create_mask(&combos))
            .collect();

        // combine/validate row/col masks
        for row in 0..rows.len() {
            for col in 0..cols.len() {
                if row_masks[row][col] == BOX_UNKNOWN {
                    row_masks[row][col] = col_masks[col][row];
                } else if col_masks[col][row] == BOX_UNKNOWN {
                    col_masks[col][row] = row_masks[row][col];
                } else if row_masks[row][col] != col_masks[col][row] {
                    process::exit(1);
                }
            }
        }

        // update rows
        row_combos = row_combos
            .par_iter()
            .zip(&row_masks)
            .map(|(combos, masks)| filter_with_mask(&combos, &masks))
            .collect();

        // update cols
        col_combos = col_combos
            .par_iter()
            .zip(&col_masks)
            .map(|(combos, masks)| filter_with_mask(&combos, &masks))
            .collect();

        for row_mask in &row_masks {
            let mut line = "".to_string();
            for ch in row_mask {
                if *ch == BOX_UNKNOWN {
                    done_solving = false;
                }
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
    let mut new_combos = Vec::<Vec<char>>::new();
    let length = combos[0].len();

    for combo in combos {
        let mut valid = true;

        for ii in 0..length {
            if mask[ii] != BOX_UNKNOWN && combo[ii] != mask[ii] {
                valid = false;
                break;
            }
        }

        if valid {
            new_combos.push(combo.to_vec());
        }
    }

    new_combos
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

fn simple_test() {
    let mut combos = create_combos(&[1, 2], 5);

    for combo in &combos {
        println!("{:?}", combo);
    }

    let mut mask = create_mask(&combos);
    println!();
    println!("mask:");
    println!("{:?}", mask);
    println!();

    mask[0] = BOX_FILLED;
    combos = filter_with_mask(&combos, &mask);

    for combo in &combos {
        println!("{:?}", combo);
    }

    let mask = create_mask(&combos);
    println!();
    println!("mask:");
    println!("{:?}", mask);
    println!();
}

fn main() {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut cols: Vec<Vec<usize>> = Vec::new();

    // Skull

    // rows.push(vec![3, 3]);
    // rows.push(vec![2, 2]);
    // rows.push(vec![2, 2]);
    // rows.push(vec![2, 2, 2, 2]);
    // rows.push(vec![2, 2, 2, 2]);
    // rows.push(vec![3, 1, 3]);
    // rows.push(vec![1, 2, 2, 1]);
    // rows.push(vec![5]);
    // rows.push(vec![2, 1, 2]);
    // rows.push(vec![4, 4]);
    // rows.push(vec![2, 1, 2]);
    // rows.push(vec![5]);
    // rows.push(vec![1, 7, 1]);

    // cols.push(vec![7, 3, 1]);
    // cols.push(vec![6, 3]);
    // cols.push(vec![1, 2, 1, 1]);
    // cols.push(vec![2, 2, 1, 2]);
    // cols.push(vec![2, 1, 2]);
    // cols.push(vec![1, 2, 3]);
    // cols.push(vec![2, 1, 2]);
    // cols.push(vec![2, 2, 1, 2]);
    // cols.push(vec![1, 2, 1, 1]);
    // cols.push(vec![6, 3]);
    // cols.push(vec![7, 3, 1]);

    // http://www.nonograms.org/nonograms/i/3541
    rows.push(vec![9]);
    rows.push(vec![16]);
    rows.push(vec![3, 2, 9]);
    rows.push(vec![2, 2, 8]);
    rows.push(vec![2, 1, 1, 8]);
    rows.push(vec![2, 3, 3, 6]);
    rows.push(vec![1, 3, 3, 6, 2]);
    rows.push(vec![1, 5, 4, 4, 1]);
    rows.push(vec![1, 7, 5, 4, 2]);
    rows.push(vec![2, 23, 1]);
    rows.push(vec![1, 2, 4, 7, 4, 2]);
    rows.push(vec![2, 2, 2, 4, 3, 1]);
    rows.push(vec![2, 2, 3, 3, 1, 2, 1]);
    rows.push(vec![2, 1, 2, 2, 5]);
    rows.push(vec![1, 1, 2, 1, 1, 2]);
    rows.push(vec![2, 1, 1, 2, 1, 1, 2]);
    rows.push(vec![4, 1, 1, 2, 1, 1, 1]);
    rows.push(vec![3, 2, 2, 2, 1, 1]);
    rows.push(vec![2, 4, 2, 2, 2, 3, 1]);
    rows.push(vec![1, 3, 2, 1, 3, 1, 1]);
    rows.push(vec![1, 1, 1, 1, 2, 2, 1, 4, 2]);
    rows.push(vec![1, 1, 1, 1, 1, 1, 1, 1, 4, 4]);
    rows.push(vec![1, 1, 3, 1, 1, 2, 1, 1, 4, 1]);
    rows.push(vec![3, 1, 1, 2, 1, 1, 1, 1, 3]);
    rows.push(vec![3, 5, 2, 2, 1, 1, 2, 2, 2, 1]);
    rows.push(vec![1, 2, 3, 1, 1, 1, 1, 1, 1, 1, 2, 2]);
    rows.push(vec![2, 2, 4, 1, 7, 1, 1, 1, 3]);
    rows.push(vec![2, 1, 2, 3, 1, 3, 1]);
    rows.push(vec![1, 2, 6, 3, 5, 2, 4]);
    rows.push(vec![3, 1, 5, 1, 3, 4, 1, 6]);

    cols.push(vec![4, 3, 1]);
    cols.push(vec![2, 1, 4]);
    cols.push(vec![1, 1, 2, 1]);
    cols.push(vec![5, 2, 1, 4]);
    cols.push(vec![2, 3, 1, 10]);
    cols.push(vec![2, 2, 1, 2, 2]);
    cols.push(vec![2, 4, 2, 2, 1, 1]);
    cols.push(vec![1, 6, 3, 1, 2]);
    cols.push(vec![2, 5, 2, 2, 1, 3, 1]);
    cols.push(vec![1, 6, 6, 2, 1, 1, 2]);
    cols.push(vec![2, 1, 3, 3, 1, 2]);
    cols.push(vec![2, 1, 3, 3, 3, 2]);
    cols.push(vec![2, 1, 3, 3]);
    cols.push(vec![3, 2, 3, 5]);
    cols.push(vec![4, 1, 6, 6, 2]);
    cols.push(vec![2, 1, 1, 1, 9, 1]);
    cols.push(vec![3, 1, 1, 1, 2, 2, 1, 1, 1]);
    cols.push(vec![3, 4, 4, 2]);
    cols.push(vec![3, 1, 5, 3, 4]);
    cols.push(vec![3, 1, 6, 4, 1]);
    cols.push(vec![3, 2, 3, 4, 2, 2]);
    cols.push(vec![4, 1, 3, 4, 1, 3, 1]);
    cols.push(vec![4, 1, 2, 5, 2]);
    cols.push(vec![4, 2, 2, 5, 2]);
    cols.push(vec![8, 1, 6]);
    cols.push(vec![7, 2, 1]);
    cols.push(vec![3, 4, 4, 3, 2]);
    cols.push(vec![2, 5, 1, 1, 4]);
    cols.push(vec![2, 3, 6, 5]);
    cols.push(vec![2, 4, 1, 3, 2, 3]);
    cols.push(vec![3, 4, 3, 3, 2, 1]);
    cols.push(vec![3, 1, 1, 3, 1, 2]);
    cols.push(vec![5, 3, 2, 2]);
    cols.push(vec![3, 2, 3, 2]);
    cols.push(vec![1, 6, 2]);

    solve_puzzle(rows, cols);

    //simple_test();
}
