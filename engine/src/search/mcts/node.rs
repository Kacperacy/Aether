use aether_core::Move;

/// MCTS Node structure
pub struct MctsNode {
    /// The move that led to this node (None for root)
    pub mv: Option<Move>,
    /// Child nodes
    pub children: Vec<MctsNode>,
    /// Number of times this node has been visited
    pub visits: u32,
    /// Total value accumulated (from perspective of player to move at parent)
    pub total_value: f64,
    /// Moves that haven't been expanded yet
    pub untried_moves: Vec<Move>,
}

impl MctsNode {
    /// Create a new node with untried moves
    pub fn new(mv: Option<Move>, untried_moves: Vec<Move>) -> Self {
        Self {
            mv,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
            untried_moves,
        }
    }

    /// Create a root node
    pub fn root(legal_moves: Vec<Move>) -> Self {
        Self::new(None, legal_moves)
    }

    /// Get the average value of this node
    #[inline]
    pub fn average_value(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_value / self.visits as f64
        }
    }

    /// Check if this node is fully expanded
    #[inline]
    pub fn is_fully_expanded(&self) -> bool {
        self.untried_moves.is_empty()
    }

    /// Check if this node is a leaf (no children)
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Standard UCB1 formula for selection
    /// UCB1 = Q + c * sqrt(ln(N_parent) / N)
    /// Where Q is average value, N_parent is parent visits, N is child visits
    #[inline]
    pub fn ucb1(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }

        let exploitation = -self.average_value();
        let exploration =
            exploration_constant * ((parent_visits as f64).ln() / self.visits as f64).sqrt();

        exploitation + exploration
    }

    /// Select the best child using UCB1
    pub fn select_child(&self, exploration_constant: f64) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_ucb = f64::NEG_INFINITY;

        for (idx, child) in self.children.iter().enumerate() {
            let ucb = child.ucb1(self.visits, exploration_constant);
            if ucb > best_ucb {
                best_ucb = ucb;
                best_idx = idx;
            }
        }

        Some(best_idx)
    }

    /// Get the child with the most visits (for final move selection)
    pub fn best_child_by_visits(&self) -> Option<&MctsNode> {
        self.children.iter().max_by_key(|c| c.visits)
    }

    /// Get the child with the highest average value
    #[allow(dead_code)]
    pub fn best_child_by_value(&self) -> Option<&MctsNode> {
        self.children
            .iter()
            .max_by(|a, b| a.average_value().partial_cmp(&b.average_value()).unwrap())
    }

    /// Expand the node by adding a child for one untried move
    pub fn expand(&mut self, move_idx: usize, child_legal_moves: Vec<Move>) -> &mut MctsNode {
        let mv = self.untried_moves.swap_remove(move_idx);
        let child = MctsNode::new(Some(mv), child_legal_moves);
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Backpropagate the result
    #[inline]
    pub fn backpropagate(&mut self, value: f64) {
        self.visits += 1;
        self.total_value += value;
    }
}

impl Default for MctsNode {
    fn default() -> Self {
        Self::new(None, Vec::new())
    }
}
