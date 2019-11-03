#[derive(Debug)]
pub enum Field {
    Value(u8),
    Options(Vec<u8>),
}

impl Field {
    pub fn with_all_options() -> Self {
        Field::Options(vec![1,2,3,4,5,6,7,8,9])
    }

    pub fn set(&mut self, val: u8) {
        assert!(1 <= val && val <= 9, "Invalid field value: {}", val);

        *self = Field::Value(val);
    }

    pub fn remove_option(&mut self, val: u8) {
        if let Field::Options(opts) = self {
            opts.retain(|&x| x != val);
        }
    }
}

pub struct Board {
    data: Vec<Field>,
    changed: bool,
    steps: Option<Vec<(u8, u8)>>,
}

impl Board {
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(9 * 9);
        for _ in 0..9 * 9 {
            data.push(Field::with_all_options());
        }

        Board {
            data,
            changed: false,
            steps: None,
        }
    }

    fn neighbours(pos: (usize, usize)) -> Vec<(usize, usize)> {
        let (row, col) = pos;
        let mut ret = Vec::with_capacity(9 + 9 + 9 - 4 - 3);

        for c in 0..9 {
            if c != col {
                ret.push((row, c));
            }
        }

        for r in 0..9 {
            if r != row {
                ret.push((r, col));
            }
        }

        let square_base_row = 3 * (row / 3);
        let square_base_col = 3 * (col / 3);

        for r in square_base_row .. square_base_row + 3 {
            if r == row {
                continue;
            }

            for c in square_base_col .. square_base_col + 3 {
                if c != col {
                    ret.push((r, c));
                }
            }
        }

        ret
    }

    pub fn record_steps(&mut self, enable: bool) {
        self.steps = if enable { Some(Vec::new()) } else { None };
    }

    #[allow(dead_code)]
    pub fn field(&self, pos: (usize, usize)) -> &Field {
        &self.data[pos.0 * 9 + pos.1]
    }

    pub fn fields(&self) -> &[Field] {
        &self.data
    }

    pub fn steps(&self) -> &Option<Vec<(u8, u8)>> {
        &self.steps
    }

    fn set_idx(&mut self, idx: usize, val: u8) {
        self.data[idx].set(val);

        for pos in Self::neighbours((idx / 9, idx % 9)) {
            self.data[pos.0 * 9 + pos.1].remove_option(val);
        }

        if let Some(ref mut steps) = self.steps {
            steps.push( (idx as u8, val) );
        }

        self.changed = true;
    }

    #[allow(dead_code)]
    pub fn set(&mut self, pos: (usize, usize), val: u8) {
        self.set_idx(pos.0 * 9 + pos.1, val)
    }

    pub fn fill(&mut self, data: impl Iterator<Item = Option<u8>>) {
        data
            .take(self.data.len())
            .enumerate()
            .filter_map(|(i, e)| e.map(|x| (i, x)))
            .for_each(|(i, val)| self.set_idx(i, val as u8));
    }

    fn solve_sole_option(&mut self) {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(idx, fld)| match fld {
                Field::Options(opts) if opts.len() == 1 => Some((idx, opts[0])),
                _ => None,
            })
            .collect::<Vec<_>>()
            .iter()
            .for_each(|(idx, val)| self.set_idx(*idx, *val));
    }

    fn solve_by_neighbourhood(
        &mut self, positions: impl Iterator<Item = usize>
    ) {
        let mut list : [Vec<_>; 9] = Default::default();

        for idx in positions {
            if let Field::Options(opts) = &self.data[idx] {
                for num in opts {
                    list[*num as usize - 1].push(idx);
                }
            }
        }

        for (num, e) in list.iter().enumerate() {
            let num = num as u8 + 1;

            match e.len() {
                0 => (),
                1 => self.set_idx(e[0], num),
                _ => {
                    let mut it = e.iter();
                    let (row, col) = match it.next().unwrap() {
                        x => (x / 9, x % 9),
                    };

                    let mut sole_row = true;
                    let mut sole_col = true;
                    while let Some(x) = it.next() {
                        if x / 9 != row {
                            sole_row = false;
                        }

                        if x % 9 != col {
                            sole_col = false;
                        }
                    }

                    if sole_row {
                        (0..9).map(|c| row * 9 + c)
                            .filter(|idx| !e.contains(&idx))
                            .for_each(|idx| self.data[idx].remove_option(num));
                    }

                    if sole_col {
                        (0..9).map(|r| r * 9 + col)
                            .filter(|idx| !e.contains(&idx))
                            .for_each(|idx| self.data[idx].remove_option(num));
                    }
                }
            }
        }
    }

    pub fn solve(&mut self) {
        loop {
            self.changed = false;
            self.solve_sole_option();

            for row in 0..9 {
                self.solve_by_neighbourhood((0..9).map(|c| row * 9 + c));
            }

            for col in 0..9 {
                self.solve_by_neighbourhood((0..9).map(|r| r * 9 + col));
            }

            for square_row in 0..3 {
                for square_col in 0..3 {
                    let square_base_row = 3 * square_row;
                    let square_base_col = 3 * square_col;

                    self.solve_by_neighbourhood(
                        (square_base_row .. square_base_row + 3).flat_map(|r| {
                            (square_base_col .. square_base_col + 3)
                                .map(move |c| r * 9 + c)
                        })
                    );
                }
            }

            if !self.changed {
                break;
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::board_from_string;

    impl Board {
        fn to_num_vec(&self) -> Vec<u8> {
            self.fields().iter().map(|x| match x {
                Field::Options(_) => 0,
                Field::Value(v) => *v,
            }).collect::<Vec<_>>()
        }
    }

    #[test]
    fn neighbours_0_0() {
        assert_eq!(
            Board::neighbours((0, 0)),
            vec![
                (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (0, 8),
                (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0),
                (1, 1), (1, 2), (2, 1), (2, 2),
            ]
        );
    }

    #[test]
    fn neighbours_1_1() {
        assert_eq!(
            Board::neighbours((1, 1)),
            vec![
                (1, 0), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6), (1, 7), (1, 8),
                (0, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1),
                (0, 0), (0, 2), (2, 0), (2, 2),
            ]
        );
    }

    #[test]
    fn neighbours_5_5() {
        assert_eq!(
            Board::neighbours((5, 5)),
            vec![
                (5, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 6), (5, 7), (5, 8),
                (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (6, 5), (7, 5), (8, 5),
                (3, 3), (3, 4), (4, 3), (4, 4),
            ]
        );
    }

    #[test]
    fn neighbours_6_2() {
        assert_eq!(
            Board::neighbours((6, 2)),
            vec![
                (6, 0), (6, 1), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8),
                (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (7, 2), (8, 2),
                (7, 0), (7, 1), (8, 0), (8, 1),
            ]
        );
    }

    #[test]
    fn solve_hard() {
        // https://sudoku.tagesspiegel.de/sudoku-sehr-schwer/
        let mut board = board_from_string(
            "92.   ...   ...\
             5..   87.   ...\
             .38   .91   ...\

             .52   93.   16.\
             .9.   ...   .3.\
             .73   .64   98.\

             ...   41.   25.\
             ...   .53   ..1\
             ...   ...   .73"
        );
        board.solve();

        assert_eq!(
            board.to_num_vec(),
            vec![
                9, 2, 6,    3, 4, 5,    7, 1, 8,
                5, 4, 1,    8, 7, 2,    3, 9, 6,
                7, 3, 8,    6, 9, 1,    4, 2, 5,

                8, 5, 2,    9, 3, 7,    1, 6, 4,
                6, 9, 4,    1, 2, 8,    5, 3, 7,
                1, 7, 3,    5, 6, 4,    9, 8, 2,

                3, 8, 7,    4, 1, 6,    2, 5, 9,
                2, 6, 9,    7, 5, 3,    8, 4, 1,
                4, 1, 5,    2, 8, 9,    6, 7, 3,
            ]
        );
    }

    #[test]
    fn solve_normal() {
        // https://sudoku.tagesspiegel.de/
        let mut board = board_from_string(
            "..4   ..5  .2.\
             .52   .36  84.\
             .16   .82  ...\

             2..   .5.  4..\
             ...   .1.  73.\
             641   ...  ..8\

             ...   8..  ..7\
             12.   ...  ..4\
             7..   ...  1.9"
        );
        board.solve();

        assert_eq!(
            board.to_num_vec(),
            vec![
                8, 7, 4,    1, 9, 5,    6, 2, 3,
                9, 5, 2,    7, 3, 6,    8, 4, 1,
                3, 1, 6,    4, 8, 2,    9, 7, 5,

                2, 3, 7,    9, 5, 8,    4, 1, 6,
                5, 8, 9,    6, 1, 4,    7, 3, 2,
                6, 4, 1,    3, 2, 7,    5, 9, 8,

                4, 9, 3,    8, 6, 1,    2, 5, 7,
                1, 2, 8,    5, 7, 9,    3, 6, 4,
                7, 6, 5,    2, 4, 3,    1, 8, 9,
            ]
        );
    }

    #[test]
    #[ignore]
    fn solve_very_hard() {
        // https://sudoku.zeit.de/sudoku-sehr-schwer 26.10.2019
        let mut board = board_from_string(
            "4..   8..   3..\
             59.   ..2   7..\
             3..   574   ...\

             9..   6..   28-\
             6..   ,,5   1..\
             81.   4..   ,,,\

             ,,,   ..9   ,,2\
             28.   ,,,   .16\
             .4.   ,,,   ..."
        );
        board.solve();

        assert_eq!(
            board.to_num_vec(),
            vec![
                4, 2, 7,    8, 9, 6,    3, 5, 1,
                5, 9, 8,    3, 1, 2,    7, 6, 4,
                3, 6, 1,    5, 7, 4,    9, 2, 8,

                9, 7, 4,    6, 3, 1,    2, 8, 5,
                6, 3, 2,    9, 8, 5,    1, 4, 7,
                8, 1, 5,    4, 2, 7,    6, 9, 3,

                7, 5, 6,    1, 4, 9,    8, 3, 2,
                2, 8, 9,    7, 5, 3,    4, 1, 6,
                1, 4, 3,    2, 6, 8,    5, 7, 9,
            ]
        );
    }

    #[test]
    #[ignore]
    fn solve_very_hard_2() {
        // http://opensudoku.moire.org/#about-puzzles
        let mut board = board_from_string(
            "...   9..   2.3\
             .26   ..3   .8.\
             83.   7..   ...\

             5.3   ..1   6..\
             ...   .3.   ...\
             ..2   5..   8.9\

             ...   ..7   .61\
             .6.   3..   47.\
             7.4   ..6   ..."
        );
        board.solve();

        assert_eq!(
            board.to_num_vec(),
            vec![
                1, 4, 7,   9, 6, 8,   2, 5, 3,
                9, 2, 6,   1, 5, 3,   7, 8, 4,
                8, 3, 5,   7, 4, 2,   9, 1, 6,

                5, 9, 3,   2, 8, 1,   6, 4, 7,
                4, 7, 8,   6, 3, 9,   1, 2, 5,
                6, 1, 2,   5, 7, 4,   8, 3, 9,

                3, 8, 9,   4, 2, 7,   5, 6, 1,
                2, 6, 1,   3, 9, 5,   4, 7, 8,
                7, 5, 4,   8, 1, 6,   3, 9, 2,
            ]
        );
    }

    #[test]
    fn solve_by_neighbourhood() {
        let mut board = board_from_string(
            "..8   ...   ...\
             914   536   ...\
             657   ..8   ...\

             ...   2..   ...\
             ...   ...   ...\
             ...   ...   ..."
        );
        board.solve();

        // because you know 2 must be on L1C1 or L1C2 it can not be on L1C4
        // and hence, must be on L3C4

        assert_eq!(
            board.to_num_vec(),
            vec![
                0, 0, 8,   0, 0, 0,   0, 0, 0,
                9, 1, 4,   5, 3, 6,   0, 0, 0,
                6, 5, 7,   0, 2, 8,   0, 0, 0,

                0, 0, 0,   2, 0, 0,   0, 0, 0,
                0, 0, 0,   0, 0, 0,   0, 0, 0,
                0, 0, 0,   0, 0, 0,   0, 0, 0,

                0, 0, 0,   0, 0, 0,   0, 0, 0,
                0, 0, 0,   0, 0, 0,   0, 0, 0,
                0, 0, 0,   0, 0, 0,   0, 0, 0,
            ]
        );
    }
}
