fn solve(islands: &[usize], row_length: usize) -> usize {
    let mut islands_count = 0;
    let mut row1 = &islands[0..row_length];
    let mut i = 0;
    while i < row_length {
        if row1[i] == 1 {
            while i < row_length && row1[i] == 1 {
                i += 1;
            }
            islands_count += 1;
        }
        i += 1;
    }

    let mut next_row_start = row_length;
    while next_row_start < islands.len() {
        let row2 = &islands[next_row_start..next_row_start + row_length];
        let mut i = 0;
        while i < row_length {
            if row2[i] == 1 {
                let mut is_island = true;
                while i < row_length && row2[i] == 1 {
                    is_island = is_island && row1[i] == 0;
                    i += 1;
                }
                if is_island {
                    islands_count += 1;
                }
            }
            i += 1;
        }
        row1 = row2;
        next_row_start += row_length;
    }

    islands_count
}