// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A compressed type identifier is more efficient to use than a `TypeId`, but only be used for up to 256 types.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CompressedTypeIdentifier(u8);

impl CompressedTypeIdentifier
{
	const ExclusiveMaximum: usize = u8::MAX as usize;

	#[inline(always)]
	fn index(self) -> usize
	{
		self.0 as usize
	}
	
	#[inline(always)]
	fn next<A: Array>(array: &ArrayVec<A>) -> Self
	{
		let length = array.len();
		debug_assert_ne!(length, A::CAPACITY, "No more space available");
		Self(length as u8)
	}
}
