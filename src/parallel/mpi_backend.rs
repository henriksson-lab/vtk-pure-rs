//! MPI backend for distributed operations (feature-gated behind `mpi`).
//!
//! Requires: `apt install libopenmpi-dev`
//!
//! Wraps the `mpi` crate to provide distributed gather, scatter,
//! and ghost exchange for vtk-rs data types.

use mpi::topology::Communicator;
use mpi::traits::*;

use crate::data::PolyData;
use crate::parallel::decomposition::Partition;

/// Initialize MPI and return the world communicator info.
pub struct MpiContext {
    universe: mpi::environment::Universe,
}

impl MpiContext {
    /// Initialize MPI. Call once at program start.
    pub fn init() -> Result<Self, String> {
        let universe = mpi::initialize()
            .ok_or_else(|| "MPI already initialized or unavailable".to_string())?;
        Ok(Self { universe })
    }

    /// Get the rank (process ID) of this process.
    pub fn rank(&self) -> usize {
        self.universe.world().rank() as usize
    }

    /// Get the total number of processes.
    pub fn size(&self) -> usize {
        self.universe.world().size() as usize
    }

    /// Barrier: wait for all processes.
    pub fn barrier(&self) {
        self.universe.world().barrier();
    }

    /// Broadcast a byte buffer from rank 0 to all.
    pub fn broadcast_bytes(&self, data: &mut Vec<u8>) {
        let world = self.universe.world();
        let root = world.process_at_rank(0);

        // First broadcast the length
        let mut len = data.len() as u64;
        if world.rank() == 0 {
            root.broadcast_into(&mut len);
        } else {
            root.broadcast_into(&mut len);
            data.resize(len as usize, 0);
        }

        // Then broadcast the data
        root.broadcast_into(data.as_mut_slice());
    }

    /// Gather f64 values from all ranks to rank 0.
    pub fn gather_f64(&self, local_value: f64) -> Vec<f64> {
        let world = self.universe.world();
        let root = world.process_at_rank(0);
        if world.rank() == 0 {
            let mut gathered = vec![0.0f64; world.size() as usize];
            root.gather_into_root(&local_value, &mut gathered);
            gathered
        } else {
            root.gather_into(&local_value);
            vec![]
        }
    }

    /// All-reduce sum of f64 across all ranks.
    pub fn allreduce_sum(&self, local: f64) -> f64 {
        let world = self.universe.world();
        let mut global = 0.0f64;
        world.all_reduce_into(&local, &mut global, mpi::collective::SystemOperation::sum());
        global
    }

    /// All-reduce max of f64 across all ranks.
    pub fn allreduce_max(&self, local: f64) -> f64 {
        let world = self.universe.world();
        let mut global = 0.0f64;
        world.all_reduce_into(&local, &mut global, mpi::collective::SystemOperation::max());
        global
    }

    /// Send a serialized partition to another rank.
    pub fn send_partition_size(&self, dest: usize, num_points: usize, num_cells: usize) {
        let world = self.universe.world();
        let data = [num_points as u64, num_cells as u64];
        world.process_at_rank(dest as i32).send(&data[..]);
    }

    /// Receive partition size from another rank.
    pub fn recv_partition_size(&self, source: usize) -> (usize, usize) {
        let world = self.universe.world();
        let (msg, _status) = world.process_at_rank(source as i32).receive_vec::<u64>();
        (msg[0] as usize, msg[1] as usize)
    }
}
