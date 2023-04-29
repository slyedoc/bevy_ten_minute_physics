use bevy::prelude::*;

pub struct SpatialHash {
    pub spacing: f32,
    pub table_size: usize,
    pub cell_start: Vec<usize>,
    pub cell_entries: Vec<usize>,
    pub query_ids: Vec<usize>,
    pub query_size: usize,
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self::new(1.0, 1000)
    }
}

impl SpatialHash {
    pub fn new(spacing: f32, max_num_objects: usize) -> Self {
        let table_size = 2 * max_num_objects;
        Self {
            spacing,
            table_size,
            cell_start: vec![0; table_size + 1],
            cell_entries: vec![0; max_num_objects],
            query_ids: vec![0; max_num_objects],
            query_size: 0,
        }
    }

    pub fn hash_coords(&self, xi: usize, yi: usize, zi: usize) -> usize {
        let h = (xi * 92837111) ^ (yi * 689287499) ^ (zi * 283923481); // fantasy function
        h % self.table_size
    }

    pub fn int_coord(&self, coord: f32) -> usize {
        return (coord / self.spacing).floor() as usize;
    }

    pub fn hash_pos(&self, position: Vec3) -> usize {
        self.hash_coords(
            self.int_coord(position.x),
            self.int_coord(position.y),
            self.int_coord(position.z),
        )
    }

    pub fn create(&mut self, pos: &[Vec3]) {
        let num_objects = pos.len().min(self.cell_entries.len());        

        // determine cell sizes
        self.cell_start.fill(0);
        self.cell_entries.fill(0);

        for i in 0..num_objects {
            let h = self.hash_pos(pos[i]);
            self.cell_start[h] += 1;
        }

        // determine cells starts
        let mut start = 0;
        for i in 0..self.table_size {
            start += self.cell_start[i];
            self.cell_start[i] = start;
        }
        self.cell_start[self.table_size] = start; // guard

        // fill in objects ids
        for i in 0..num_objects {
            let h = self.hash_pos(pos[i]);
            self.cell_start[h] -= 1;
            self.cell_entries[self.cell_start[h]] = i;
        }
    }

    pub fn query(&mut self, pos: &[f32], nr: usize, max_dist: f32) {
        let x0 = self.int_coord(pos[0 + nr] - max_dist);
        let y0 = self.int_coord(pos[1 + nr] - max_dist);
        let z0 = self.int_coord(pos[2 + nr] - max_dist);

        let x1 = self.int_coord(pos[0 + nr] + max_dist);
        let y1 = self.int_coord(pos[1 + nr] + max_dist);
        let z1 = self.int_coord(pos[2 + nr] + max_dist);

        self.query_size = 0;

        for xi in x0..=x1 {
            for yi in y0..=y1 {
                for zi in z0..=z1 {
                    let h = self.hash_coords(xi, yi, zi);
                    let start = self.cell_start[h];
                    let end = self.cell_start[h + 1];
                    for i in start..end {
                        self.query_ids[self.query_size] = self.cell_entries[i];
                        self.query_size += 1;
                    }
                }
            }
        }
    }
}
