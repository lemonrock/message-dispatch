// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Represents a message to be enqueued to a buffer.
#[derive(Debug)]
#[repr(C)]
pub(super) struct Message
{
	message_header: MessageHeader,
	padding_then_message_body_then_padding: VariablySizedPaddingThenAVariablySizedMessageBodyThenVariablySizedPadding,
}

impl Message
{
	#[inline(always)]
	pub(super) fn enqueue<FixedSizeMessageBody: Sized>(magic_ring_buffer: &MagicRingBuffer, fixed_size_message_body_compressed_type_identifier: CompressedTypeIdentifier, fixed_size_message_body_constructor: impl FnOnce(NonNull<FixedSizeMessageBody>))
	{
		let largest_possible_total_message_size_including_message_header = Self::largest_possible_total_message_size_including_message_header::<FixedSizeMessageBody>();

		magic_ring_buffer.write_some_data(largest_possible_total_message_size_including_message_header, |buffer_sized_as_for_maximum_possible|
		{
			Self::enqueue_once_buffer_allocated::<FixedSizeMessageBody, _>(buffer_sized_as_for_maximum_possible, fixed_size_message_body_compressed_type_identifier, fixed_size_message_body_constructor)
		})
	}
	
	#[inline(always)]
	pub(super) fn smallest_possible_total_message_size_including_message_header() -> usize
	{
		Self::largest_possible_total_message_size_including_message_header::<()>()
	}

	/// NOTE: In Rust, alignment is *always* a positive power of two (ie never zero), is 1 for packed structs and is never less than the struct's size, either.
	/// An empty struct by default has an alignment of 1 but it too can have any legal alignment.
	///
	/// Should, after monomorphization and compiler optimization, become nothing more than a constant value.
	#[inline(always)]
	pub(super) fn largest_possible_total_message_size_including_message_header<FixedSizeMessageBody: Sized>() -> usize
	{
		const MessageHeaderSize: usize = size_of::<MessageHeader>();
		const MessageHeaderAlignment: usize = align_of::<MessageHeader>();
		let MessageBodySize = size_of::<FixedSizeMessageBody>();
		let MessageBodyAlignment = align_of::<FixedSizeMessageBody>();

		if MessageBodyAlignment > MessageHeaderAlignment
		{
			let maximum_padding_after_message_header_but_before_message_body = MessageBodyAlignment - MessageHeaderAlignment;
			let largest_possible_total_message_size_including_message_header = MessageHeaderSize + maximum_padding_after_message_header_but_before_message_body + MessageBodySize;

			largest_possible_total_message_size_including_message_header
		}
		else
		{
			let takes_up_no_space = MessageBodySize == 0;

			if takes_up_no_space
			{
				MessageHeaderSize
			}
			else
			{
				MessageHeaderSize * 2
			}
		}
	}

	/// Enqueues a new message into the `buffer_sized_as_for_maximum_possible` if there is space available.
	///
	/// Assumes the `buffer_sized_as_for_maximum_possible` is correctly aligned for a `MessageHeader`.
	#[inline(always)]
	fn enqueue_once_buffer_allocated<FixedSizeMessageBody: Sized, FixedSizeMessageBodyConstructor: FnOnce(NonNull<FixedSizeMessageBody>)>(buffer_sized_as_for_maximum_possible: &mut [u8], fixed_size_message_body_compressed_type_identifier: CompressedTypeIdentifier, fixed_size_message_body_constructor: FixedSizeMessageBodyConstructor)
	{
		let total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after = buffer_sized_as_for_maximum_possible.len();
		debug_assert_eq!(Self::largest_possible_total_message_size_including_message_header::<FixedSizeMessageBody>(), total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after, "buffer_sized_as_for_maximum_possible is not");
		debug_assert!(total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after < u16::MAX as usize, "message is far too large");

		let buffer_pointer = buffer_sized_as_for_maximum_possible.as_ptr() as usize;
		debug_assert_eq!(buffer_pointer % align_of::<MessageHeader>(), 0, "buffer_sized_as_for_maximum_possible is not correctly aligned for a MessageHeader");

		const MessageHeaderSize: usize = size_of::<MessageHeader>();
		const MessageHeaderAlignment: usize = align_of::<MessageHeader>();
		let MessageBodySize = size_of::<FixedSizeMessageBody>();
		let MessageBodyAlignment = align_of::<FixedSizeMessageBody>();

		let (message_body_pointer, number_of_bytes_padding_to_align_message_body) = if MessageBodyAlignment > MessageHeaderAlignment
		{
			let first_possible_message_body_pointer = buffer_pointer + MessageHeaderSize;
			let message_body_pointer = round_up_to_alignment::<FixedSizeMessageBody>(first_possible_message_body_pointer);
			(message_body_pointer, message_body_pointer - first_possible_message_body_pointer)
		}
		else
		{
			let takes_up_no_space = MessageBodySize == 0;

			let actual_total_message_size = if takes_up_no_space
			{
				MessageHeaderSize
			}
			else
			{
				MessageHeaderSize * 2
			};
			(actual_total_message_size, 0)
		};

		unsafe
		{
			let message_header = &mut * (buffer_pointer as *mut MessageHeader);
			write(&mut message_header.compressed_type_identifier, fixed_size_message_body_compressed_type_identifier);
			write(&mut message_header.number_of_bytes_padding_to_align_message_body, number_of_bytes_padding_to_align_message_body as u8); // TODO: Could be stored as `SQRT(MessageContentsAlignment)`, thus allowing more alignments, at the cost of more processing when dequeued.
			write(&mut message_header.total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after, total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after as u16); // TODO: Could be stored less message header size and leading padding, thus allowing a little more data.
		}

		fixed_size_message_body_constructor(unsafe { NonNull::new_unchecked(message_body_pointer as *mut FixedSizeMessageBody) })
	}

	/// Returns `(next_message_pointer, R)`.
	#[inline(always)]
	pub(super) fn process_next_message_in_buffer<R, MessageProcessor: FnMut(CompressedTypeIdentifier, NonNull<VariablySizedMessageBody>) -> R>(buffer: &mut [u8], mut message_processor: MessageProcessor) -> (usize, R)
	{
		const MessageHeaderSize: usize = size_of::<MessageHeader>();
		const MessageHeaderAlignment: usize = align_of::<MessageHeader>();

		let buffer_pointer = buffer.as_mut_ptr() as usize;
		let buffer_length = buffer.len();
		debug_assert_eq!(buffer_pointer % MessageHeaderAlignment, 0, "Buffer is not aligned on a MessageHeader");
		debug_assert!(buffer_length >= MessageHeaderSize, "Buffer is too small to contain a MessageHeader");

		let message_header = unsafe { &mut * (buffer_pointer as *mut MessageHeader) };

		let total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after = message_header.total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after();

		debug_assert!(buffer_length >= total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after, "Buffer is too small to contain the Message");
		debug_assert_eq!((buffer_pointer + total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after) % MessageHeaderAlignment, 0, "Message is not aligned such that the next MessageHeader is aligned");

		let message_body_compressed_type_identifier = message_header.compressed_type_identifier;
		let variably_sized_message_body = message_header.variably_sized_message_body();

		let outcome = message_processor(message_body_compressed_type_identifier, variably_sized_message_body);
		(total_message_size_including_message_header_padding_to_align_before_message_body_and_padding_to_align_after, outcome)
	}
}
