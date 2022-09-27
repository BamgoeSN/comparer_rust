use std::{cmp::Reverse, collections::BinaryHeap};

use rand::Rng;

/// Represents a tree with undirected edges, with no root set.
pub struct Tree {
    // The number of nodes
    n: usize,
    // A list of UNDIRECTED edges of the tree
    edges: Vec<(usize, usize)>,
}

impl Tree {
    /// Returns a reference to the edge list.
    pub fn get_edges(&self) -> &[(usize, usize)] {
        &self.edges
    }

    /// Generates a tree from a given prufer sequence.
    pub fn from_prufer(prufer: &[usize]) -> Option<Self> {
        // A random tree is generated using Prüfer Sequence
        // Reference: https://www.secmem.org/blog/2019/10/20/Pr%C3%BCfer-sequence/
        let n = prufer.len() + 2;
        if prufer.iter().any(|&a| a >= n) {
            return None;
        }

        let mut edges: Vec<(usize, usize)> = Vec::with_capacity(2 * n - 2);
        let mut degree: Vec<u32> = vec![1; n];

        for &a in prufer {
            degree[a] += 1;
        }

        let mut leaves: BinaryHeap<Reverse<usize>> =
            (0..n).filter(|&i| degree[i] == 1).map(Reverse).collect();

        for &a in prufer {
            let u = leaves.pop().unwrap().0;
            degree[u] -= 1;
            degree[a] -= 1;
            edges.push((u, a));
            if degree[a] == 1 {
                leaves.push(Reverse(a));
            }
        }

        let u = leaves.pop().unwrap().0;
        let v = leaves.pop().unwrap().0;
        edges.push((u, v));

        Some(Self { n, edges })
    }

    /// Generates a random tree with n edges using Prüfer Sequence.
    pub fn random_tree(n: usize, rng: &mut impl Rng) -> Self {
        if n <= 1 {
            return Self { n, edges: vec![] };
        } else if n == 2 {
            return Self {
                n,
                edges: vec![(0, 1)],
            };
        }

        let prufer: Vec<_> = (0..n - 2).map(|_| rng.gen_range(0..n)).collect();
        Self::from_prufer(&prufer).unwrap()
    }

    /// Returns a parent list where parent[i] is Some(parent of i-th node).
    /// parent[root] is always None.
    /// The function might panic if the tree instance represents an invalid graph,
    /// which shouldn't happen if only public methods are used.
    pub fn set_root(&self, root: usize) -> Vec<Option<usize>> {
        let mut graph: Vec<Vec<usize>> = vec![vec![]; self.n];
        for &(u, v) in &self.edges {
            graph[u].push(v);
            graph[v].push(u);
        }

        let mut parent = vec![None; self.n];
        Self::dfs(root, self.n, &graph, &mut parent);
        parent
    }

    fn dfs(curr: usize, prev: usize, graph: &[Vec<usize>], parent: &mut [Option<usize>]) {
        for &next in &graph[curr] {
            if next == prev {
                continue;
            }
            parent[next] = Some(curr);
            Self::dfs(next, curr, graph, parent);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Tree;

    #[test]
    fn generate_random_tree() {
        use rand::thread_rng;

        let n: usize = 100000;
        let tree = Tree::random_tree(n, &mut thread_rng());
        let mut uf = UnionFind::new(n);
        for &(u, v) in tree.get_edges() {
            assert!(!uf.is_reachable(u, v));
            uf.union(u, v);
        }
    }

    struct UnionFind {
        size: usize,
        parents: Vec<usize>,
        group_size: Vec<usize>,
        group_num: usize,
    }

    impl UnionFind {
        /// Returns a new UnionFind instance where `size` number of elements are in their own disjoint set.
        fn new(size: usize) -> Self {
            Self {
                size,
                parents: vec![size; size],
                group_size: vec![1; size],
                group_num: size,
            }
        }

        fn find_root(&mut self, x: usize) -> usize {
            if self.parents[x] == self.size {
                return x;
            }
            let root = self.find_root(self.parents[x]);
            self.parents[x] = root;
            root
        }

        /// Returns true if there exists a path from a to b.
        fn is_reachable(&mut self, a: usize, b: usize) -> bool {
            self.find_root(a) == self.find_root(b)
        }

        /// Add an edge between a and b.
        fn union(&mut self, a: usize, b: usize) {
            let a_root = self.find_root(a);
            let b_root = self.find_root(b);

            if a_root != b_root {
                self.group_num -= 1;
                let a_size = self.group_size[a_root];
                let b_size = self.group_size[b_root];
                if a_size < b_size {
                    self.parents[a_root] = b_root;
                    self.group_size[b_root] += a_size;
                } else {
                    self.parents[b_root] = a_root;
                    self.group_size[a_root] += b_size;
                }
            }
        }
    }
}
