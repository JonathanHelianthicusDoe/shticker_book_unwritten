//! The code in this module has been adapted more or less line-for-line from
//! the source of bsdiff 4.3, which is written in C and is here "oxidized" on a
//! very basic level, even retaining the original variable names, and is not
//! (yet) idiomatic or optimized or anything like that.
//!
//! bsdiff4 is licensed under a slight variation of the FreeBSD license, which
//! requires that the licensing text be reproduced alongside any modified or
//! unmodified redistributions. So here it is (the same text can be found in
//! the LICENSE.bsdiff4 file):
//!
//! ```c
//! /*-
//!  * Copyright 2003-2005 Colin Percival
//!  * All rights reserved
//!  *
//!  * Redistribution and use in source and binary forms, with or without
//!  * modification, are permitted providing that the following conditions
//!  * are met:
//!  * 1. Redistributions of source code must retain the above copyright
//!  *    notice, this list of conditions and the following disclaimer.
//!  * 2. Redistributions in binary form must reproduce the above copyright
//!  *    notice, this list of conditions and the following disclaimer in the
//!  *    documentation and/or other materials provided with the distribution.
//!  *
//!  * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
//!  * IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
//!  * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
//!  * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
//!  * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//!  * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
//!  * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//!  * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
//!  * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING
//!  * IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
//!  * POSSIBILITY OF SUCH DAMAGE.
//!  */
//! ```

use crate::{error::Error, util};
use bzip2::read::BzDecoder as BzReadDecoder;
use std::{
    self,
    ffi::{OsStr, OsString},
    io::{prelude::*, Seek, SeekFrom},
    path::Path,
};

pub fn patch_file<P: AsRef<Path>, Q: AsRef<Path>>(
    patch_file_path: P,
    target_file_path: Q,
) -> Result<(), Error> {
    let target_file_osstr: &OsStr = target_file_path.as_ref().as_ref();
    let mut temp_file_path =
        OsString::with_capacity(target_file_osstr.len() + ".tmp".len());
    temp_file_path.push(target_file_path.as_ref());
    temp_file_path.push(".tmp");

    bsdiff_patch(patch_file_path, &target_file_path, &temp_file_path)?;

    std::fs::rename(&temp_file_path, &target_file_path)
        .map_err(Error::FileRenameError)?;

    Ok(())
}

fn bsdiff_patch<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    patch_file_path: P,
    old_file_path: Q,
    new_file_path: R,
) -> Result<(), Error> {
    let new = apply_patch(patch_file_path, old_file_path)?;

    // Write the new file
    let mut fd = util::create_file(&new_file_path)?;
    fd.write_all(&new[..]).map_err(|ioe| {
        Error::FileWriteError(new_file_path.as_ref().to_path_buf(), ioe)
    })?;

    Ok(())
}

fn apply_patch<P: AsRef<Path>, Q: AsRef<Path>>(
    patch_file_path: P,
    old_file_path: Q,
) -> Result<Vec<u8>, Error> {
    let header = {
        // Open patch file
        let mut f = util::open_file(&patch_file_path)?;

        /*
         * File format:
         *
         *   | offset | len | data
         * --+--------+-----+----------------------
         *   | 0      | 8   | "BSDIFF40"
         *   | 8      | 8   | X
         *   | 16     | 8   | Y
         *   | 24     | 8   | sizeof(new_file)
         *   | 32     | X   | bzip2(control block)
         *   | 32+X   | Y   | bzip2(diff block)
         *   | 32+X+Y | ??? | bzip2(extra block)
         *
         * With control block a set of triples (x, y, z) meaning "add x bytes
         * from old_file to x bytes from the diff block; copy y bytes from the
         * extra block; seek forwards in old_file by z bytes".
         */

        // Read header
        let mut header = [0u8; 32];
        f.read_exact(&mut header).map_err(|ioe| {
            Error::FileReadError(patch_file_path.as_ref().to_path_buf(), ioe)
        })?;

        header
    };

    // Check for appropriate magic
    if header[..8] != b"BSDIFF40"[..] {
        return Err(Error::BadPatchVersion);
    }

    // Read lengths from header
    let bzctrllen = offtin(&header[8..]);
    let bzdatalen = offtin(&header[16..]);
    let newsize = offtin(&header[24..]);
    if bzctrllen < 0 || bzdatalen < 0 || newsize < 0 {
        return Err(Error::BadPatchSize);
    }

    // Open patch file in the right places with libbzip2
    let mut cpf = util::open_file(&patch_file_path)?;
    cpf.seek(SeekFrom::Start(32)).map_err(Error::SeekError)?;
    let mut cpfbz2 = BzReadDecoder::new(cpf);
    let mut dpf = util::open_file(&patch_file_path)?;
    dpf.seek(SeekFrom::Start((32 + bzctrllen) as u64))
        .map_err(Error::SeekError)?;
    let mut dpfbz2 = BzReadDecoder::new(dpf);
    let mut epf = util::open_file(&patch_file_path)?;
    epf.seek(SeekFrom::Start((32 + bzctrllen + bzdatalen) as u64))
        .map_err(Error::SeekError)?;
    let mut epfbz2 = BzReadDecoder::new(epf);

    let mut fd = util::open_file(&old_file_path)?;
    let oldsize = fd.seek(SeekFrom::End(0)).map_err(Error::SeekError)? as i64;
    let mut old = Vec::with_capacity(oldsize as usize);
    old.resize_with(oldsize as usize, Default::default);
    fd.seek(SeekFrom::Start(0)).map_err(Error::SeekError)?;
    fd.read_exact(&mut old[..]).map_err(|ioe| {
        Error::FileReadError(old_file_path.as_ref().to_path_buf(), ioe)
    })?;

    let mut new = Vec::with_capacity(newsize as usize);
    new.resize_with(newsize as usize, Default::default);

    // Start the actual patching
    let mut buf = [0u8; 8];
    let mut ctrl = [0i64; 3];
    let mut oldpos = 0i64;
    let mut newpos = 0i64;
    while newpos < newsize {
        // Read control data
        for ctrl_off in ctrl.iter_mut() {
            cpfbz2.read_exact(&mut buf).map_err(Error::DecodeError)?;
            *ctrl_off = offtin(&buf);
        }

        // Sanity check
        if newpos + ctrl[0] > newsize {
            return Err(Error::PatchSanityCheckFail(0));
        }

        // Read diff string
        dpfbz2
            .read_exact(&mut new[newpos as usize..(newpos + ctrl[0]) as usize])
            .map_err(Error::DecodeError)?;

        // Add old data to diff string
        for i in 0..ctrl[0] {
            if (oldpos + i >= 0) && (oldpos + i < oldsize) {
                let new_index = (newpos + i) as usize;
                let old_index = (oldpos + i) as usize;
                new[new_index] = new[new_index].wrapping_add(old[old_index]);
            }
        }

        // Adjust pointers
        newpos += ctrl[0];
        oldpos += ctrl[0];

        // Sanity check
        if newpos + ctrl[1] > newsize {
            return Err(Error::PatchSanityCheckFail(1));
        }

        // Read extra string
        epfbz2
            .read_exact(&mut new[newpos as usize..(newpos + ctrl[1]) as usize])
            .map_err(Error::DecodeError)?;

        // Adjust pointers
        newpos += ctrl[1];
        oldpos += ctrl[2];
    }

    Ok(new)
}

fn offtin(buf: &[u8]) -> i64 {
    let mut y = i64::from(buf[7] & 0x7f);

    y *= 256;
    y += i64::from(buf[6]);
    y *= 256;
    y += i64::from(buf[5]);
    y *= 256;
    y += i64::from(buf[4]);
    y *= 256;
    y += i64::from(buf[3]);
    y *= 256;
    y += i64::from(buf[2]);
    y *= 256;
    y += i64::from(buf[1]);
    y *= 256;
    y += i64::from(buf[0]);

    if (buf[7] & 0x80) != 0 {
        y = -y;
    }

    y
}
