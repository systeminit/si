/// Akin to `zip` but does not halt the iterator if A or B return None. Only ends when both return None.
pub struct OptZip<A, B>
where
    A: Iterator,
    B: Iterator,
{
    a: A,
    a_none: bool,
    b: B,
    b_none: bool,
}

impl<A, B> OptZip<A, B>
where
    A: Iterator,
    B: Iterator,
{
    pub fn new(a: A, b: B) -> Self {
        Self {
            a,
            a_none: false,
            b,
            b_none: false,
        }
    }
}

impl<A, B> Iterator for OptZip<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (Option<A::Item>, Option<B::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.a_none, self.b_none) {
            (true, true) => None,
            (true, false) => {
                let b = self.b.next();
                self.b_none = b.is_none();
                if self.b_none { None } else { Some((None, b)) }
            }
            (false, true) => {
                let a = self.a.next();
                self.a_none = a.is_none();
                if self.a_none { None } else { Some((a, None)) }
            }
            (false, false) => {
                let a = self.a.next();
                let b = self.b.next();
                self.a_none = a.is_none();
                self.b_none = b.is_none();

                if self.a_none && self.b_none {
                    None
                } else {
                    Some((a, b))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OptZip;

    #[test]
    fn b_longer_than_a() {
        let a = [1, 2, 3];
        let b = [1, 2, 3, 4, 5, 6, 7];

        let opt_zip = OptZip::new(a.into_iter(), b.into_iter());

        let expected = vec![
            (Some(1), Some(1)),
            (Some(2), Some(2)),
            (Some(3), Some(3)),
            (None, Some(4)),
            (None, Some(5)),
            (None, Some(6)),
            (None, Some(7)),
        ];

        let collected: Vec<(_, _)> = opt_zip.into_iter().collect();

        assert_eq!(expected, collected);
    }

    #[test]
    fn a_longer_than_b() {
        let a = [1, 2, 3, 4];
        let b = [1, 2, 3];

        let opt_zip = OptZip::new(a.into_iter(), b.into_iter());

        let expected = vec![
            (Some(1), Some(1)),
            (Some(2), Some(2)),
            (Some(3), Some(3)),
            (Some(4), None),
        ];

        let collected: Vec<(_, _)> = opt_zip.into_iter().collect();

        assert_eq!(expected, collected);
    }

    #[test]
    fn same_size() {
        let a = [1, 2, 3, 4];
        let b = [1, 2, 3, 4];

        let opt_zip = OptZip::new(a.into_iter(), b.into_iter());

        let expected = vec![
            (Some(1), Some(1)),
            (Some(2), Some(2)),
            (Some(3), Some(3)),
            (Some(4), Some(4)),
        ];

        let collected: Vec<(_, _)> = opt_zip.into_iter().collect();

        assert_eq!(expected, collected);
    }
}
