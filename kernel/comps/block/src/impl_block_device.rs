// SPDX-License-Identifier: MPL-2.0

use ostd::mm::{Frame, FrameAllocOptions, Segment, VmIo};

use super::{
    bio::{Bio, BioEnqueueError, BioSegment, BioStatus, BioType, BioWaiter, SubmittedBio},
    id::{Bid, Sid},
    BlockDevice, BLOCK_SIZE, SECTOR_SIZE,
};
use crate::prelude::*;

/// Implements several commonly used APIs for the block device to conveniently
/// read and write block(s).
// TODO: Add API to submit bio with multiple segments in scatter/gather manner.
impl dyn BlockDevice {
    /// Synchronously reads contiguous blocks starting from the `bid`.
    pub fn read_blocks(&self, bid: Bid, segment: &Segment) -> Result<BioStatus, BioEnqueueError> {
        let bio = create_bio_from_segment(BioType::Read, bid, segment);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously reads contiguous blocks starting from the `bid`.
    pub fn read_blocks_async(
        &self,
        bid: Bid,
        segment: &Segment,
    ) -> Result<BioWaiter, BioEnqueueError> {
        let bio = create_bio_from_segment(BioType::Read, bid, segment);
        bio.submit(self)
    }

    /// Synchronously reads one block indicated by the `bid`.
    pub fn read_block(&self, bid: Bid, frame: &Frame) -> Result<BioStatus, BioEnqueueError> {
        let bio = create_bio_from_frame(BioType::Read, bid, frame);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously reads one block indicated by the `bid`.
    pub fn read_block_async(&self, bid: Bid, frame: &Frame) -> Result<BioWaiter, BioEnqueueError> {
        let bio = create_bio_from_frame(BioType::Read, bid, frame);
        bio.submit(self)
    }

    /// Synchronously writes contiguous blocks starting from the `bid`.
    pub fn write_blocks(&self, bid: Bid, segment: &Segment) -> Result<BioStatus, BioEnqueueError> {
        let bio = create_bio_from_segment(BioType::Write, bid, segment);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously writes contiguous blocks starting from the `bid`.
    pub fn write_blocks_async(
        &self,
        bid: Bid,
        segment: &Segment,
    ) -> Result<BioWaiter, BioEnqueueError> {
        let bio = create_bio_from_segment(BioType::Write, bid, segment);
        bio.submit(self)
    }

    /// Synchronously writes one block indicated by the `bid`.
    pub fn write_block(&self, bid: Bid, frame: &Frame) -> Result<BioStatus, BioEnqueueError> {
        let bio = create_bio_from_frame(BioType::Write, bid, frame);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously writes one block indicated by the `bid`.
    pub fn write_block_async(&self, bid: Bid, frame: &Frame) -> Result<BioWaiter, BioEnqueueError> {
        let bio = create_bio_from_frame(BioType::Write, bid, frame);
        bio.submit(self)
    }
}

impl VmIo for dyn BlockDevice {
    /// Reads consecutive bytes of several sectors in size.
    fn read_bytes(&self, offset: usize, buf: &mut [u8]) -> ostd::Result<()> {
        if offset % SECTOR_SIZE != 0 || buf.len() % SECTOR_SIZE != 0 {
            return Err(ostd::Error::InvalidArgs);
        }
        if buf.is_empty() {
            return Ok(());
        }

        let (bio, bio_segment) = {
            let num_blocks = {
                let first = Bid::from_offset(offset).to_raw();
                let last = Bid::from_offset(offset + buf.len() - 1).to_raw();
                last - first + 1
            };
            let segment = FrameAllocOptions::new(num_blocks as usize)
                .uninit(true)
                .alloc_contiguous()?;
            let bio_segment = BioSegment::from_segment(segment, offset % BLOCK_SIZE, buf.len());

            (
                Bio::new(
                    BioType::Read,
                    Sid::from_offset(offset),
                    vec![bio_segment.clone()],
                    None,
                ),
                bio_segment,
            )
        };

        let status = bio.submit_and_wait(self)?;
        match status {
            BioStatus::Complete => {
                let _ = bio_segment.reader().read(&mut buf.into());
                Ok(())
            }
            _ => Err(ostd::Error::IoError),
        }
    }

    /// Writes consecutive bytes of several sectors in size.
    fn write_bytes(&self, offset: usize, buf: &[u8]) -> ostd::Result<()> {
        if offset % SECTOR_SIZE != 0 || buf.len() % SECTOR_SIZE != 0 {
            return Err(ostd::Error::InvalidArgs);
        }
        if buf.is_empty() {
            return Ok(());
        }

        let bio = {
            let num_blocks = {
                let first = Bid::from_offset(offset).to_raw();
                let last = Bid::from_offset(offset + buf.len() - 1).to_raw();
                last - first + 1
            };
            let segment = FrameAllocOptions::new(num_blocks as usize)
                .uninit(true)
                .alloc_contiguous()?;
            segment.write_bytes(offset % BLOCK_SIZE, buf)?;
            let len = segment
                .writer()
                .skip(offset % BLOCK_SIZE)
                .write(&mut buf.into());
            let bio_segment = BioSegment::from_segment(segment, offset % BLOCK_SIZE, len);
            Bio::new(
                BioType::Write,
                Sid::from_offset(offset),
                vec![bio_segment],
                None,
            )
        };

        let status = bio.submit_and_wait(self)?;
        match status {
            BioStatus::Complete => Ok(()),
            _ => Err(ostd::Error::IoError),
        }
    }
}

impl dyn BlockDevice {
    /// Asynchronously writes consecutive bytes of several sectors in size.
    pub fn write_bytes_async(&self, offset: usize, buf: &[u8]) -> ostd::Result<BioWaiter> {
        if offset % SECTOR_SIZE != 0 || buf.len() % SECTOR_SIZE != 0 {
            return Err(ostd::Error::InvalidArgs);
        }
        if buf.is_empty() {
            return Ok(BioWaiter::new());
        }

        let bio = {
            let num_blocks = {
                let first = Bid::from_offset(offset).to_raw();
                let last = Bid::from_offset(offset + buf.len() - 1).to_raw();
                last - first + 1
            };
            let segment = FrameAllocOptions::new(num_blocks as usize)
                .uninit(true)
                .alloc_contiguous()?;
            segment.write_bytes(offset % BLOCK_SIZE, buf)?;
            let len = segment
                .writer()
                .skip(offset % BLOCK_SIZE)
                .write(&mut buf.into());
            let bio_segment = BioSegment::from_segment(segment, offset % BLOCK_SIZE, len);
            Bio::new(
                BioType::Write,
                Sid::from_offset(offset),
                vec![bio_segment],
                Some(general_complete_fn),
            )
        };

        let complete = bio.submit(self)?;
        Ok(complete)
    }
}

// TODO: Maybe we should have a builder for `Bio`.
fn create_bio_from_segment(type_: BioType, bid: Bid, segment: &Segment) -> Bio {
    let bio_segment = BioSegment::from_segment(segment.clone(), 0, segment.nbytes());
    Bio::new(
        type_,
        Sid::from(bid),
        vec![bio_segment],
        Some(general_complete_fn),
    )
}

// TODO: Maybe we should have a builder for `Bio`.
fn create_bio_from_frame(type_: BioType, bid: Bid, frame: &Frame) -> Bio {
    let bio_segment = BioSegment::from_frame(frame.clone(), 0, BLOCK_SIZE);
    Bio::new(
        type_,
        Sid::from(bid),
        vec![bio_segment],
        Some(general_complete_fn),
    )
}

fn general_complete_fn(bio: &SubmittedBio) {
    match bio.status() {
        BioStatus::Complete => (),
        err_status => log::error!(
            "failed to do {:?} on the device with error status: {:?}",
            bio.type_(),
            err_status
        ),
    }
}
