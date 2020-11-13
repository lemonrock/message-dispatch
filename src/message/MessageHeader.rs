// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


#[derive(Debug)]
#[repr(C)]
struct MessageHeader
{
	compressed_type_identifier: CompressedTypeIdentifier,
	number_of_bytes_padding_to_align_message_body: u8,
	total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after: u16,
}

impl MessageHeader
{
	#[inline(always)]
	fn variably_sized_message_body(&mut self) -> NonNull<VariablySizedMessageBody>
	{
		new_non_null(self.message_body_pointer() as *mut VariablySizedMessageBody)
	}

	#[inline(always)]
	fn total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after(&self) -> usize
	{
		self.total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after as usize
	}

	#[inline(always)]
	fn message_body_pointer(&self) -> usize
	{
		self.base_pointer() + self.number_of_bytes_padding_to_align_message_body()
	}

	#[inline(always)]
	fn base_pointer(&self) -> usize
	{
		self as *const Self as usize
	}

	#[inline(always)]
	fn number_of_bytes_padding_to_align_message_body(&self) -> usize
	{
		self.number_of_bytes_padding_to_align_message_body as usize
	}
}
