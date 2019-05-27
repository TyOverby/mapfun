use std::collections::btree_map::*;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Point(f64, f64);

impl Eq for Point {}
impl Ord for Point {
    fn cmp(&self, other: &Point) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn inline_map<A: Sized, B: Sized, F: Fn(A) -> B>(mut input: Vec<A>, f: F) -> Vec<B> {
    use std::mem::*;
    assert!(size_of::<A>() == size_of::<B>());
    assert!(align_of::<A>() == align_of::<B>());
    for item in input.iter_mut() {
        let mut swap_into: A = unsafe { uninitialized() };
        swap(&mut swap_into, item);

        let mut out: B = f(swap_into);
        let out_ref: &mut B = &mut out;
        let out_transmuted: &mut A = unsafe { transmute(out_ref) };
        swap(out_transmuted, item);
        forget(out);
    }
    let result: Vec<B> = unsafe { transmute(input) };
    result
}

fn dedup(line_segments: Vec<Vec<(f64, f64)>>) -> Vec<Vec<(f64, f64)>> {
    let mut into = inline_map(line_segments, |ls| inline_map(ls, |(a, b)| Point(a, b)));
    dedup_inner(&mut into);
    let outof = inline_map(into, |ls| inline_map(ls, |Point(a, b)| (a, b)));
    outof
}

// Deduplicates the list of line segments.  The order may not be the same after processing.
fn dedup_inner(line_segments: &mut Vec<Vec<Point>>) {
    let mut start_points = BTreeMap::new();
    let mut should_remove = vec![];

    for (i, line_segment) in line_segments.iter().enumerate() {
        if line_segment.is_empty() {
            should_remove.push(i);
            continue;
        }
        let my_start = line_segment[0];

        match start_points.entry(my_start) {
            Entry::Vacant(vacant) => {
                vacant.insert(vec![i]);
            }
            Entry::Occupied(mut occupied) => {
                let collided = occupied
                    .get()
                    .iter()
                    .any(|idx| &line_segments[*idx] == line_segment);
                if collided {
                    should_remove.push(i);
                } else {
                    occupied.get_mut().push(i);
                }
            }
        }
    }

    should_remove.sort();
    for &idx in should_remove.iter().rev() {
        line_segments.swap_remove(idx);
    }
}

#[test]
fn empty_list() {
    let mut input = vec![];
    dedup_inner(&mut input);
    assert_eq!(Vec::<Vec<Point>>::new(), input)
}


#[test]
fn one_empty_segment() {
    let mut input = vec![vec![]];
    dedup_inner(&mut input);
    assert_eq!(Vec::<Vec<Point>>::new(), input)
}

#[test]
fn one_full_segment() {
    let mut input = vec![vec![Point(0.0, 0.0), Point(1.0, 1.0)]];
    dedup_inner(&mut input);
    assert_eq!(vec![vec![Point(0.0, 0.0), Point(1.0, 1.0)]], input)
}

#[test]
fn one_duplicate_segment() {
    let mut input = vec![
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
    ];
    dedup_inner(&mut input);
    assert_eq!(vec![vec![Point(0.0, 0.0), Point(1.0, 1.0)]], input)
}

#[test]
fn one_duplicate_segment_with_another_segment_inbetween() {
    let mut input = vec![
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
        vec![Point(2.0, 3.0), Point(4.0, 5.0)],
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
    ];
    dedup_inner(&mut input);
    assert_eq!(
        vec![
            vec![Point(0.0, 0.0), Point(1.0, 1.0)],
            vec![Point(2.0, 3.0), Point(4.0, 5.0)],
        ],
        input
    )
}

#[test]
fn two_duplicate_segments() {
    let mut input = vec![
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
        vec![Point(2.0, 3.0), Point(4.0, 5.0)],
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
        vec![Point(2.0, 3.0), Point(4.0, 5.0)],
    ];
    dedup_inner(&mut input);
    assert_eq!(
        vec![
            vec![Point(0.0, 0.0), Point(1.0, 1.0)],
            vec![Point(2.0, 3.0), Point(4.0, 5.0)],
        ],
        input
    )
}


#[test]
fn two_distinct_segments_that_start_with_the_same_point() {
    let mut input = vec![
        vec![Point(0.0, 0.0), Point(1.0, 1.0)],
        vec![Point(0.0, 0.0), Point(4.0, 5.0)],
    ];
    dedup_inner(&mut input);
    assert_eq!(
        vec![
            vec![Point(0.0, 0.0), Point(1.0, 1.0)],
            vec![Point(0.0, 0.0), Point(4.0, 5.0)],
        ],
        input
    )
}
