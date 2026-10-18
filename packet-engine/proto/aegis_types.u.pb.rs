const _: () = ::protobuf::__internal::assert_compatible_gencode_version("0.36.0-release");
// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__Empty_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct Empty {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<Empty>
}

impl ::protobuf::Message for Empty {
  type MessageView<'msg> = EmptyView<'msg>;
  type MessageMut<'msg> = EmptyMut<'msg>;
}

impl ::std::default::Default for Empty {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for Empty {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `Empty` is `Sync` because it does not implement interior mutability.
//    Neither does `EmptyMut`.
unsafe impl ::std::marker::Sync for Empty {}

// SAFETY:
// - `Empty` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for Empty {}

impl ::protobuf::Proxied for Empty {
  type View<'msg> = EmptyView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for Empty {}

impl ::protobuf::MutProxied for Empty {
  type Mut<'msg> = EmptyMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct EmptyView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, Empty>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for EmptyView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for EmptyView<'msg> {
  type Message = Empty;
}

impl ::std::fmt::Debug for EmptyView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for EmptyView<'_> {
  fn default() -> EmptyView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, Empty>> for EmptyView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, Empty>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> EmptyView<'msg> {

  pub fn to_owned(&self) -> Empty {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

}

// SAFETY:
// - `EmptyView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for EmptyView<'_> {}

// SAFETY:
// - `EmptyView` is `Send` because while its alive a `EmptyMut` cannot.
// - `EmptyView` does not use thread-local data.
unsafe impl ::std::marker::Send for EmptyView<'_> {}

impl<'msg> ::protobuf::AsView for EmptyView<'msg> {
  type Proxied = Empty;
  fn as_view(&self) -> ::protobuf::View<'msg, Empty> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for EmptyView<'msg> {
  fn into_view<'shorter>(self) -> EmptyView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<Empty> for EmptyView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> Empty {
    let mut dst = Empty::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<Empty> for EmptyMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> Empty {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for Empty {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for EmptyView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for EmptyMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct EmptyMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, Empty>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for EmptyMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for EmptyMut<'msg> {
  type Message = Empty;
}

impl ::std::fmt::Debug for EmptyMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, Empty>> for EmptyMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, Empty>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> EmptyMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, Empty> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> Empty {
    ::protobuf::AsView::as_view(self).to_owned()
  }

}

// SAFETY:
// - `EmptyMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for EmptyMut<'_> {}

// SAFETY:
// - `EmptyMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for EmptyMut<'_> {}

impl<'msg> ::protobuf::AsView for EmptyMut<'msg> {
  type Proxied = Empty;
  fn as_view(&self) -> ::protobuf::View<'_, Empty> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for EmptyMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, Empty>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for EmptyMut<'msg> {
  type MutProxied = Empty;
  fn as_mut(&mut self) -> EmptyMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for EmptyMut<'msg> {
  fn into_mut<'shorter>(self) -> EmptyMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl Empty {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, Empty> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> EmptyView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> EmptyMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

}  // impl Empty

impl ::std::ops::Drop for Empty {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for Empty {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for Empty {
  type Proxied = Self;
  fn as_view(&self) -> EmptyView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for Empty {
  type MutProxied = Self;
  fn as_mut(&mut self) -> EmptyMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for Empty {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__Empty_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__Empty_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__Empty_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for Empty {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for Empty {
  type Msg = Empty;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<Empty> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for Empty {
  type Msg = Empty;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<Empty> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for EmptyMut<'_> {
  type Msg = Empty;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<Empty> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for EmptyMut<'_> {
  type Msg = Empty;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<Empty> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for EmptyView<'_> {
  type Msg = Empty;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<Empty> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for EmptyMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__ScanCommand_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct ScanCommand {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<ScanCommand>
}

impl ::protobuf::Message for ScanCommand {
  type MessageView<'msg> = ScanCommandView<'msg>;
  type MessageMut<'msg> = ScanCommandMut<'msg>;
}

impl ::std::default::Default for ScanCommand {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for ScanCommand {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `ScanCommand` is `Sync` because it does not implement interior mutability.
//    Neither does `ScanCommandMut`.
unsafe impl ::std::marker::Sync for ScanCommand {}

// SAFETY:
// - `ScanCommand` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for ScanCommand {}

impl ::protobuf::Proxied for ScanCommand {
  type View<'msg> = ScanCommandView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for ScanCommand {}

impl ::protobuf::MutProxied for ScanCommand {
  type Mut<'msg> = ScanCommandMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ScanCommandView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ScanCommand>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ScanCommandView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for ScanCommandView<'msg> {
  type Message = ScanCommand;
}

impl ::std::fmt::Debug for ScanCommandView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for ScanCommandView<'_> {
  fn default() -> ScanCommandView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, ScanCommand>> for ScanCommandView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ScanCommand>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ScanCommandView<'msg> {

  pub fn to_owned(&self) -> ScanCommand {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // command_id: optional uint64
  pub fn command_id(self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        0, (0u64).into()
      ).try_into().unwrap()
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // port: optional uint32
  pub fn port(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // priority: optional double
  pub fn priority(self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        4, (0f64).into()
      ).try_into().unwrap()
    }
  }

  // source: optional enum aegis.types.TargetSource
  pub fn source(self) -> super::TargetSource {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::TargetSource::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // conditional_probability: optional double
  pub fn conditional_probability(self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        6, (0f64).into()
      ).try_into().unwrap()
    }
  }

  // asn: optional uint32
  pub fn asn(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // feature_tuple: repeated uint32
  pub fn feature_tuple(self) -> ::protobuf::RepeatedView<'msg, u32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        8
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<u32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

  // generated_at: optional string
  pub fn generated_at(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cidr_block: optional string
  pub fn cidr_block(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // asn_name: optional string
  pub fn asn_name(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `ScanCommandView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for ScanCommandView<'_> {}

// SAFETY:
// - `ScanCommandView` is `Send` because while its alive a `ScanCommandMut` cannot.
// - `ScanCommandView` does not use thread-local data.
unsafe impl ::std::marker::Send for ScanCommandView<'_> {}

impl<'msg> ::protobuf::AsView for ScanCommandView<'msg> {
  type Proxied = ScanCommand;
  fn as_view(&self) -> ::protobuf::View<'msg, ScanCommand> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ScanCommandView<'msg> {
  fn into_view<'shorter>(self) -> ScanCommandView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<ScanCommand> for ScanCommandView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ScanCommand {
    let mut dst = ScanCommand::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<ScanCommand> for ScanCommandMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ScanCommand {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for ScanCommand {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ScanCommandView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ScanCommandMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct ScanCommandMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ScanCommand>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ScanCommandMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for ScanCommandMut<'msg> {
  type Message = ScanCommand;
}

impl ::std::fmt::Debug for ScanCommandMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, ScanCommand>> for ScanCommandMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ScanCommand>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ScanCommandMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, ScanCommand> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> ScanCommand {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // command_id: optional uint64
  pub fn command_id(&self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        0, (0u64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_command_id(&mut self, val: u64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u64_at_index(
        0, val.into()
      )
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // port: optional uint32
  pub fn port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // priority: optional double
  pub fn priority(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        4, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_priority(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        4, val.into()
      )
    }
  }

  // source: optional enum aegis.types.TargetSource
  pub fn source(&self) -> super::TargetSource {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::TargetSource::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_source(&mut self, val: super::TargetSource) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        5, val.into()
      )
    }
  }

  // conditional_probability: optional double
  pub fn conditional_probability(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        6, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_conditional_probability(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        6, val.into()
      )
    }
  }

  // asn: optional uint32
  pub fn asn(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_asn(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        7, val.into()
      )
    }
  }

  // feature_tuple: repeated uint32
  pub fn feature_tuple(&self) -> ::protobuf::RepeatedView<'_, u32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        8
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<u32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn feature_tuple_mut(&mut self) -> ::protobuf::RepeatedMut<'_, u32> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        8,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_feature_tuple(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<u32>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        src);
    }
  }

  // generated_at: optional string
  pub fn generated_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_generated_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // cidr_block: optional string
  pub fn cidr_block(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cidr_block(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

  // asn_name: optional string
  pub fn asn_name(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_asn_name(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

}

// SAFETY:
// - `ScanCommandMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for ScanCommandMut<'_> {}

// SAFETY:
// - `ScanCommandMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for ScanCommandMut<'_> {}

impl<'msg> ::protobuf::AsView for ScanCommandMut<'msg> {
  type Proxied = ScanCommand;
  fn as_view(&self) -> ::protobuf::View<'_, ScanCommand> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ScanCommandMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, ScanCommand>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for ScanCommandMut<'msg> {
  type MutProxied = ScanCommand;
  fn as_mut(&mut self) -> ScanCommandMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for ScanCommandMut<'msg> {
  fn into_mut<'shorter>(self) -> ScanCommandMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl ScanCommand {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, ScanCommand> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> ScanCommandView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> ScanCommandMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // command_id: optional uint64
  pub fn command_id(&self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        0, (0u64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_command_id(&mut self, val: u64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u64_at_index(
        0, val.into()
      )
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // port: optional uint32
  pub fn port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // priority: optional double
  pub fn priority(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        4, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_priority(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        4, val.into()
      )
    }
  }

  // source: optional enum aegis.types.TargetSource
  pub fn source(&self) -> super::TargetSource {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::TargetSource::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_source(&mut self, val: super::TargetSource) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        5, val.into()
      )
    }
  }

  // conditional_probability: optional double
  pub fn conditional_probability(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        6, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_conditional_probability(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        6, val.into()
      )
    }
  }

  // asn: optional uint32
  pub fn asn(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_asn(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        7, val.into()
      )
    }
  }

  // feature_tuple: repeated uint32
  pub fn feature_tuple(&self) -> ::protobuf::RepeatedView<'_, u32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        8
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<u32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn feature_tuple_mut(&mut self) -> ::protobuf::RepeatedMut<'_, u32> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        8,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_feature_tuple(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<u32>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        src);
    }
  }

  // generated_at: optional string
  pub fn generated_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_generated_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // cidr_block: optional string
  pub fn cidr_block(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cidr_block(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

  // asn_name: optional string
  pub fn asn_name(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_asn_name(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

}  // impl ScanCommand

impl ::std::ops::Drop for ScanCommand {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for ScanCommand {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for ScanCommand {
  type Proxied = Self;
  fn as_view(&self) -> ScanCommandView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for ScanCommand {
  type MutProxied = Self;
  fn as_mut(&mut self) -> ScanCommandMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for ScanCommand {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__ScanCommand_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$N,P0P)P.P P.P P)P=1X1X1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__ScanCommand_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__ScanCommand_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ScanCommand {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ScanCommand {
  type Msg = ScanCommand;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ScanCommand> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ScanCommand {
  type Msg = ScanCommand;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ScanCommand> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ScanCommandMut<'_> {
  type Msg = ScanCommand;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ScanCommand> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ScanCommandMut<'_> {
  type Msg = ScanCommand;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ScanCommand> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ScanCommandView<'_> {
  type Msg = ScanCommand;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ScanCommand> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ScanCommandMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__L4Response_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct L4Response {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<L4Response>
}

impl ::protobuf::Message for L4Response {
  type MessageView<'msg> = L4ResponseView<'msg>;
  type MessageMut<'msg> = L4ResponseMut<'msg>;
}

impl ::std::default::Default for L4Response {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for L4Response {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `L4Response` is `Sync` because it does not implement interior mutability.
//    Neither does `L4ResponseMut`.
unsafe impl ::std::marker::Sync for L4Response {}

// SAFETY:
// - `L4Response` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for L4Response {}

impl ::protobuf::Proxied for L4Response {
  type View<'msg> = L4ResponseView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for L4Response {}

impl ::protobuf::MutProxied for L4Response {
  type Mut<'msg> = L4ResponseMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct L4ResponseView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, L4Response>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for L4ResponseView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for L4ResponseView<'msg> {
  type Message = L4Response;
}

impl ::std::fmt::Debug for L4ResponseView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for L4ResponseView<'_> {
  fn default() -> L4ResponseView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, L4Response>> for L4ResponseView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, L4Response>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> L4ResponseView<'msg> {

  pub fn to_owned(&self) -> L4Response {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // event_id: optional string
  pub fn event_id(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // target_ip: optional bytes
  pub fn target_ip(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // target_port: optional uint32
  pub fn target_port(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // result: optional enum aegis.types.L4Response.L4Result
  pub fn result(self) -> super::l4_response::L4Result {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        4, (super::l4_response::L4Result::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // rtt_usec: optional uint32
  pub fn rtt_usec(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // ttl: optional uint32
  pub fn ttl(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        6, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // tcp_window: optional uint32
  pub fn tcp_window(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // source_pop: optional string
  pub fn source_pop(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // sent_at: optional string
  pub fn sent_at(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // received_at: optional string
  pub fn received_at(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `L4ResponseView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for L4ResponseView<'_> {}

// SAFETY:
// - `L4ResponseView` is `Send` because while its alive a `L4ResponseMut` cannot.
// - `L4ResponseView` does not use thread-local data.
unsafe impl ::std::marker::Send for L4ResponseView<'_> {}

impl<'msg> ::protobuf::AsView for L4ResponseView<'msg> {
  type Proxied = L4Response;
  fn as_view(&self) -> ::protobuf::View<'msg, L4Response> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for L4ResponseView<'msg> {
  fn into_view<'shorter>(self) -> L4ResponseView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<L4Response> for L4ResponseView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> L4Response {
    let mut dst = L4Response::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<L4Response> for L4ResponseMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> L4Response {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for L4Response {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for L4ResponseView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for L4ResponseMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct L4ResponseMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, L4Response>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for L4ResponseMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for L4ResponseMut<'msg> {
  type Message = L4Response;
}

impl ::std::fmt::Debug for L4ResponseMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, L4Response>> for L4ResponseMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, L4Response>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> L4ResponseMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, L4Response> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> L4Response {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // result: optional enum aegis.types.L4Response.L4Result
  pub fn result(&self) -> super::l4_response::L4Result {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        4, (super::l4_response::L4Result::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_result(&mut self, val: super::l4_response::L4Result) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        4, val.into()
      )
    }
  }

  // rtt_usec: optional uint32
  pub fn rtt_usec(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_rtt_usec(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        5, val.into()
      )
    }
  }

  // ttl: optional uint32
  pub fn ttl(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        6, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_ttl(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        6, val.into()
      )
    }
  }

  // tcp_window: optional uint32
  pub fn tcp_window(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_tcp_window(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        7, val.into()
      )
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // sent_at: optional string
  pub fn sent_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sent_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // received_at: optional string
  pub fn received_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_received_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

}

// SAFETY:
// - `L4ResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for L4ResponseMut<'_> {}

// SAFETY:
// - `L4ResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for L4ResponseMut<'_> {}

impl<'msg> ::protobuf::AsView for L4ResponseMut<'msg> {
  type Proxied = L4Response;
  fn as_view(&self) -> ::protobuf::View<'_, L4Response> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for L4ResponseMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, L4Response>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for L4ResponseMut<'msg> {
  type MutProxied = L4Response;
  fn as_mut(&mut self) -> L4ResponseMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for L4ResponseMut<'msg> {
  fn into_mut<'shorter>(self) -> L4ResponseMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl L4Response {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, L4Response> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> L4ResponseView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> L4ResponseMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // result: optional enum aegis.types.L4Response.L4Result
  pub fn result(&self) -> super::l4_response::L4Result {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        4, (super::l4_response::L4Result::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_result(&mut self, val: super::l4_response::L4Result) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        4, val.into()
      )
    }
  }

  // rtt_usec: optional uint32
  pub fn rtt_usec(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_rtt_usec(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        5, val.into()
      )
    }
  }

  // ttl: optional uint32
  pub fn ttl(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        6, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_ttl(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        6, val.into()
      )
    }
  }

  // tcp_window: optional uint32
  pub fn tcp_window(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        7, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_tcp_window(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        7, val.into()
      )
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // sent_at: optional string
  pub fn sent_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sent_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // received_at: optional string
  pub fn received_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_received_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

}  // impl L4Response

impl ::std::ops::Drop for L4Response {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for L4Response {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for L4Response {
  type Proxied = Self;
  fn as_view(&self) -> L4ResponseView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for L4Response {
  type MutProxied = Self;
  fn as_mut(&mut self) -> L4ResponseMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for L4Response {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__L4Response_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$1X0P)P.P.P)P)P)P1X1X1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__L4Response_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__L4Response_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for L4Response {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for L4Response {
  type Msg = L4Response;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L4Response> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L4Response {
  type Msg = L4Response;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L4Response> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for L4ResponseMut<'_> {
  type Msg = L4Response;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L4Response> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L4ResponseMut<'_> {
  type Msg = L4Response;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L4Response> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L4ResponseView<'_> {
  type Msg = L4Response;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L4Response> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for L4ResponseMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

pub mod l4_response {
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct L4Result(i32);

#[allow(non_upper_case_globals)]
impl L4Result {
  pub const Unspecified: L4Result = L4Result(0);
  pub const Open: L4Result = L4Result(1);
  pub const Closed: L4Result = L4Result(2);
  pub const Filtered: L4Result = L4Result(3);
  pub const HostAlive: L4Result = L4Result(4);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "Open",
      2 => "Closed",
      3 => "Filtered",
      4 => "HostAlive",
      _ => return None
    })
  }
}

impl ::std::convert::From<L4Result> for i32 {
  fn from(val: L4Result) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for L4Result {
  fn from(val: i32) -> L4Result {
    Self(val)
  }
}

impl ::std::default::Default for L4Result {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for L4Result {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "L4Result::{}", constant_name)
    } else {
      write!(f, "L4Result::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for L4Result {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for L4Result {}

impl ::protobuf::Proxied for L4Result {
  type View<'a> = L4Result;
}

impl ::protobuf::AsView for L4Result {
  type Proxied = L4Result;

  fn as_view(&self) -> L4Result {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for L4Result {
  fn into_view<'shorter>(self) -> L4Result where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for L4Result {
  const NAME: &'static str = "L4Result";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2|3|4)
  }
}

impl ::protobuf::__internal::EntityType for L4Result {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


}  // pub mod l4_response


// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__TLSFingerprint_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct TLSFingerprint {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<TLSFingerprint>
}

impl ::protobuf::Message for TLSFingerprint {
  type MessageView<'msg> = TLSFingerprintView<'msg>;
  type MessageMut<'msg> = TLSFingerprintMut<'msg>;
}

impl ::std::default::Default for TLSFingerprint {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for TLSFingerprint {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `TLSFingerprint` is `Sync` because it does not implement interior mutability.
//    Neither does `TLSFingerprintMut`.
unsafe impl ::std::marker::Sync for TLSFingerprint {}

// SAFETY:
// - `TLSFingerprint` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for TLSFingerprint {}

impl ::protobuf::Proxied for TLSFingerprint {
  type View<'msg> = TLSFingerprintView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for TLSFingerprint {}

impl ::protobuf::MutProxied for TLSFingerprint {
  type Mut<'msg> = TLSFingerprintMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct TLSFingerprintView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, TLSFingerprint>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for TLSFingerprintView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for TLSFingerprintView<'msg> {
  type Message = TLSFingerprint;
}

impl ::std::fmt::Debug for TLSFingerprintView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for TLSFingerprintView<'_> {
  fn default() -> TLSFingerprintView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, TLSFingerprint>> for TLSFingerprintView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, TLSFingerprint>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> TLSFingerprintView<'msg> {

  pub fn to_owned(&self) -> TLSFingerprint {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // event_id: optional string
  pub fn event_id(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // target_ip: optional bytes
  pub fn target_ip(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // target_port: optional uint32
  pub fn target_port(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // ja4: optional string
  pub fn ja4(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ja4s: optional string
  pub fn ja4s(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // tls_version: optional string
  pub fn tls_version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cipher_suite: optional string
  pub fn cipher_suite(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // sni: optional string
  pub fn sni(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // alpn: optional string
  pub fn alpn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // certificate_chain: optional string
  pub fn certificate_chain(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_fingerprint: optional string
  pub fn cert_fingerprint(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_sans: repeated string
  pub fn cert_sans(self) -> ::protobuf::RepeatedView<'msg, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        14
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

  // source_pop: optional string
  pub fn source_pop(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // observed_at: optional string
  pub fn observed_at(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `TLSFingerprintView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for TLSFingerprintView<'_> {}

// SAFETY:
// - `TLSFingerprintView` is `Send` because while its alive a `TLSFingerprintMut` cannot.
// - `TLSFingerprintView` does not use thread-local data.
unsafe impl ::std::marker::Send for TLSFingerprintView<'_> {}

impl<'msg> ::protobuf::AsView for TLSFingerprintView<'msg> {
  type Proxied = TLSFingerprint;
  fn as_view(&self) -> ::protobuf::View<'msg, TLSFingerprint> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for TLSFingerprintView<'msg> {
  fn into_view<'shorter>(self) -> TLSFingerprintView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<TLSFingerprint> for TLSFingerprintView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> TLSFingerprint {
    let mut dst = TLSFingerprint::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<TLSFingerprint> for TLSFingerprintMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> TLSFingerprint {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for TLSFingerprint {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for TLSFingerprintView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for TLSFingerprintMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct TLSFingerprintMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, TLSFingerprint>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for TLSFingerprintMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for TLSFingerprintMut<'msg> {
  type Message = TLSFingerprint;
}

impl ::std::fmt::Debug for TLSFingerprintMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, TLSFingerprint>> for TLSFingerprintMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, TLSFingerprint>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> TLSFingerprintMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, TLSFingerprint> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> TLSFingerprint {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // ja4: optional string
  pub fn ja4(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // ja4s: optional string
  pub fn ja4s(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4s(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        5,
        val);
    }
  }

  // tls_version: optional string
  pub fn tls_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // cipher_suite: optional string
  pub fn cipher_suite(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cipher_suite(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // sni: optional string
  pub fn sni(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sni(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // alpn: optional string
  pub fn alpn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_alpn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // certificate_chain: optional string
  pub fn certificate_chain(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_certificate_chain(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

  // cert_fingerprint: optional string
  pub fn cert_fingerprint(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_fingerprint(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_subject_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_issuer_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        13,
        val);
    }
  }

  // cert_sans: repeated string
  pub fn cert_sans(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        14
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cert_sans_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        14,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cert_sans(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        14,
        src);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        15,
        val);
    }
  }

  // observed_at: optional string
  pub fn observed_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_observed_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        16,
        val);
    }
  }

}

// SAFETY:
// - `TLSFingerprintMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for TLSFingerprintMut<'_> {}

// SAFETY:
// - `TLSFingerprintMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for TLSFingerprintMut<'_> {}

impl<'msg> ::protobuf::AsView for TLSFingerprintMut<'msg> {
  type Proxied = TLSFingerprint;
  fn as_view(&self) -> ::protobuf::View<'_, TLSFingerprint> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for TLSFingerprintMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, TLSFingerprint>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for TLSFingerprintMut<'msg> {
  type MutProxied = TLSFingerprint;
  fn as_mut(&mut self) -> TLSFingerprintMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for TLSFingerprintMut<'msg> {
  fn into_mut<'shorter>(self) -> TLSFingerprintMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl TLSFingerprint {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, TLSFingerprint> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> TLSFingerprintView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> TLSFingerprintMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // ja4: optional string
  pub fn ja4(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // ja4s: optional string
  pub fn ja4s(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4s(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        5,
        val);
    }
  }

  // tls_version: optional string
  pub fn tls_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // cipher_suite: optional string
  pub fn cipher_suite(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cipher_suite(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // sni: optional string
  pub fn sni(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sni(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // alpn: optional string
  pub fn alpn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_alpn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // certificate_chain: optional string
  pub fn certificate_chain(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        10, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_certificate_chain(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        10,
        val);
    }
  }

  // cert_fingerprint: optional string
  pub fn cert_fingerprint(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_fingerprint(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_subject_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_issuer_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        13,
        val);
    }
  }

  // cert_sans: repeated string
  pub fn cert_sans(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        14
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cert_sans_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        14,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cert_sans(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        14,
        src);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        15,
        val);
    }
  }

  // observed_at: optional string
  pub fn observed_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_observed_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        16,
        val);
    }
  }

}  // impl TLSFingerprint

impl ::std::ops::Drop for TLSFingerprint {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for TLSFingerprint {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for TLSFingerprint {
  type Proxied = Self;
  fn as_view(&self) -> TLSFingerprintView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for TLSFingerprint {
  type MutProxied = Self;
  fn as_mut(&mut self) -> TLSFingerprintMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for TLSFingerprint {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__TLSFingerprint_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$1X0P)P.P1X1X1X1X1X1X1X1X1X1XET1X1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__TLSFingerprint_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__TLSFingerprint_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for TLSFingerprint {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for TLSFingerprint {
  type Msg = TLSFingerprint;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<TLSFingerprint> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for TLSFingerprint {
  type Msg = TLSFingerprint;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<TLSFingerprint> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for TLSFingerprintMut<'_> {
  type Msg = TLSFingerprint;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<TLSFingerprint> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for TLSFingerprintMut<'_> {
  type Msg = TLSFingerprint;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<TLSFingerprint> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for TLSFingerprintView<'_> {
  type Msg = TLSFingerprint;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<TLSFingerprint> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for TLSFingerprintMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__L7Banner_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct L7Banner {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<L7Banner>
}

impl ::protobuf::Message for L7Banner {
  type MessageView<'msg> = L7BannerView<'msg>;
  type MessageMut<'msg> = L7BannerMut<'msg>;
}

impl ::std::default::Default for L7Banner {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for L7Banner {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `L7Banner` is `Sync` because it does not implement interior mutability.
//    Neither does `L7BannerMut`.
unsafe impl ::std::marker::Sync for L7Banner {}

// SAFETY:
// - `L7Banner` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for L7Banner {}

impl ::protobuf::Proxied for L7Banner {
  type View<'msg> = L7BannerView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for L7Banner {}

impl ::protobuf::MutProxied for L7Banner {
  type Mut<'msg> = L7BannerMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct L7BannerView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, L7Banner>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for L7BannerView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for L7BannerView<'msg> {
  type Message = L7Banner;
}

impl ::std::fmt::Debug for L7BannerView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for L7BannerView<'_> {
  fn default() -> L7BannerView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, L7Banner>> for L7BannerView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, L7Banner>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> L7BannerView<'msg> {

  pub fn to_owned(&self) -> L7Banner {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // event_id: optional string
  pub fn event_id(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // target_ip: optional bytes
  pub fn target_ip(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // target_port: optional uint32
  pub fn target_port(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // protocol: optional string
  pub fn protocol(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // raw_banner: optional bytes
  pub fn raw_banner(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // banner_hash: optional string
  pub fn banner_hash(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // http_method: optional string
  pub fn http_method(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // http_status_code: optional uint32
  pub fn http_status_code(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        10, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // http_server: optional string
  pub fn http_server(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_version: optional string
  pub fn ssh_version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        14, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // db_version: optional string
  pub fn db_version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // source_pop: optional string
  pub fn source_pop(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // observed_at: optional string
  pub fn observed_at(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        17, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `L7BannerView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for L7BannerView<'_> {}

// SAFETY:
// - `L7BannerView` is `Send` because while its alive a `L7BannerMut` cannot.
// - `L7BannerView` does not use thread-local data.
unsafe impl ::std::marker::Send for L7BannerView<'_> {}

impl<'msg> ::protobuf::AsView for L7BannerView<'msg> {
  type Proxied = L7Banner;
  fn as_view(&self) -> ::protobuf::View<'msg, L7Banner> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for L7BannerView<'msg> {
  fn into_view<'shorter>(self) -> L7BannerView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<L7Banner> for L7BannerView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> L7Banner {
    let mut dst = L7Banner::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<L7Banner> for L7BannerMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> L7Banner {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for L7Banner {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for L7BannerView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for L7BannerMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct L7BannerMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, L7Banner>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for L7BannerMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for L7BannerMut<'msg> {
  type Message = L7Banner;
}

impl ::std::fmt::Debug for L7BannerMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, L7Banner>> for L7BannerMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, L7Banner>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> L7BannerMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, L7Banner> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> L7Banner {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // protocol: optional string
  pub fn protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(&self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_detection_method(&mut self, val: super::DetectionMethod) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        5, val.into()
      )
    }
  }

  // raw_banner: optional bytes
  pub fn raw_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_raw_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sanitized_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // banner_hash: optional string
  pub fn banner_hash(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_banner_hash(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // http_method: optional string
  pub fn http_method(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_method(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // http_status_code: optional uint32
  pub fn http_status_code(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        10, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_http_status_code(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        10, val.into()
      )
    }
  }

  // http_server: optional string
  pub fn http_server(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_server(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // ssh_version: optional string
  pub fn ssh_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_key_exchange_algorithms(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        13,
        val);
    }
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        14, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_host_key_type(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        14,
        val);
    }
  }

  // db_version: optional string
  pub fn db_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_db_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        15,
        val);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        16,
        val);
    }
  }

  // observed_at: optional string
  pub fn observed_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        17, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_observed_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        17,
        val);
    }
  }

}

// SAFETY:
// - `L7BannerMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for L7BannerMut<'_> {}

// SAFETY:
// - `L7BannerMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for L7BannerMut<'_> {}

impl<'msg> ::protobuf::AsView for L7BannerMut<'msg> {
  type Proxied = L7Banner;
  fn as_view(&self) -> ::protobuf::View<'_, L7Banner> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for L7BannerMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, L7Banner>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for L7BannerMut<'msg> {
  type MutProxied = L7Banner;
  fn as_mut(&mut self) -> L7BannerMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for L7BannerMut<'msg> {
  fn into_mut<'shorter>(self) -> L7BannerMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl L7Banner {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, L7Banner> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> L7BannerView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> L7BannerMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // target_ip: optional bytes
  pub fn target_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_target_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // target_port: optional uint32
  pub fn target_port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_target_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        3, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        3, val.into()
      )
    }
  }

  // protocol: optional string
  pub fn protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(&self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        5, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_detection_method(&mut self, val: super::DetectionMethod) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        5, val.into()
      )
    }
  }

  // raw_banner: optional bytes
  pub fn raw_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_raw_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sanitized_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // banner_hash: optional string
  pub fn banner_hash(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        8, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_banner_hash(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        8,
        val);
    }
  }

  // http_method: optional string
  pub fn http_method(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        9, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_method(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        9,
        val);
    }
  }

  // http_status_code: optional uint32
  pub fn http_status_code(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        10, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_http_status_code(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        10, val.into()
      )
    }
  }

  // http_server: optional string
  pub fn http_server(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_server(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // ssh_version: optional string
  pub fn ssh_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        13, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_key_exchange_algorithms(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        13,
        val);
    }
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        14, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_host_key_type(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        14,
        val);
    }
  }

  // db_version: optional string
  pub fn db_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        15, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_db_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        15,
        val);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        16, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        16,
        val);
    }
  }

  // observed_at: optional string
  pub fn observed_at(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        17, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_observed_at(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        17,
        val);
    }
  }

}  // impl L7Banner

impl ::std::ops::Drop for L7Banner {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for L7Banner {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for L7Banner {
  type Proxied = Self;
  fn as_view(&self) -> L7BannerView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for L7Banner {
  type MutProxied = Self;
  fn as_mut(&mut self) -> L7BannerMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for L7Banner {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__L7Banner_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$1X0P)P.P1X.P0P1X1X1X)P1X1X1X1X1X1X1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__L7Banner_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__L7Banner_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for L7Banner {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for L7Banner {
  type Msg = L7Banner;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L7Banner> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L7Banner {
  type Msg = L7Banner;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L7Banner> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for L7BannerMut<'_> {
  type Msg = L7Banner;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L7Banner> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L7BannerMut<'_> {
  type Msg = L7Banner;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L7Banner> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for L7BannerView<'_> {
  type Msg = L7Banner;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<L7Banner> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for L7BannerMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__EnrichedHost_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct EnrichedHost {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<EnrichedHost>
}

impl ::protobuf::Message for EnrichedHost {
  type MessageView<'msg> = EnrichedHostView<'msg>;
  type MessageMut<'msg> = EnrichedHostMut<'msg>;
}

impl ::std::default::Default for EnrichedHost {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for EnrichedHost {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `EnrichedHost` is `Sync` because it does not implement interior mutability.
//    Neither does `EnrichedHostMut`.
unsafe impl ::std::marker::Sync for EnrichedHost {}

// SAFETY:
// - `EnrichedHost` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for EnrichedHost {}

impl ::protobuf::Proxied for EnrichedHost {
  type View<'msg> = EnrichedHostView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for EnrichedHost {}

impl ::protobuf::MutProxied for EnrichedHost {
  type Mut<'msg> = EnrichedHostMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct EnrichedHostView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, EnrichedHost>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for EnrichedHostView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for EnrichedHostView<'msg> {
  type Message = EnrichedHost;
}

impl ::std::fmt::Debug for EnrichedHostView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for EnrichedHostView<'_> {
  fn default() -> EnrichedHostView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, EnrichedHost>> for EnrichedHostView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, EnrichedHost>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> EnrichedHostView<'msg> {

  pub fn to_owned(&self) -> EnrichedHost {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // ip_address: optional string
  pub fn ip_address(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // host_id: optional string
  pub fn host_id(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // asn: optional uint32
  pub fn asn(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // organization: optional string
  pub fn organization(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        3, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // bgp_prefix: optional string
  pub fn bgp_prefix(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // bgp_as_path: optional string
  pub fn bgp_as_path(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // reverse_dns: optional string
  pub fn reverse_dns(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // forward_dns: optional string
  pub fn forward_dns(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // geo_tier_used: optional enum aegis.types.GeoTier
  pub fn geo_tier_used(self) -> super::GeoTier {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        8, (super::GeoTier::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // geo_latitude: optional double
  pub fn geo_latitude(self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        9, (0f64).into()
      ).try_into().unwrap()
    }
  }

  // geo_longitude: optional double
  pub fn geo_longitude(self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        10, (0f64).into()
      ).try_into().unwrap()
    }
  }

  // geo_city: optional string
  pub fn geo_city(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // geo_country: optional string
  pub fn geo_country(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // geo_confidence: optional float
  pub fn geo_confidence(self) -> f32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f32_at_index(
        13, (0f32).into()
      ).try_into().unwrap()
    }
  }

  // first_seen: optional int64
  pub fn first_seen(self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        14, (0i64).into()
      ).try_into().unwrap()
    }
  }

  // last_seen: optional int64
  pub fn last_seen(self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        15, (0i64).into()
      ).try_into().unwrap()
    }
  }

  // scan_count: optional int32
  pub fn scan_count(self) -> i32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        16, (0i32).into()
      ).try_into().unwrap()
    }
  }

  // status: optional enum aegis.types.HostStatus
  pub fn status(self) -> super::HostStatus {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        17, (super::HostStatus::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // port: optional uint32
  pub fn port(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        18, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        19, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // reclassified_protocol: optional string
  pub fn reclassified_protocol(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        20, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        21, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        22, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // banner_hash: optional string
  pub fn banner_hash(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        23, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ja4: optional string
  pub fn ja4(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        24, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ja4s: optional string
  pub fn ja4s(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        25, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // tls_version: optional string
  pub fn tls_version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        26, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cipher_suite: optional string
  pub fn cipher_suite(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        27, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // sni: optional string
  pub fn sni(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        28, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // alpn: optional string
  pub fn alpn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        29, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // http_method: optional string
  pub fn http_method(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        30, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // http_status_code: optional uint32
  pub fn http_status_code(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        31, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // http_server: optional string
  pub fn http_server(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        32, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_version: optional string
  pub fn ssh_version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        33, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        34, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        35, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_sha256: optional string
  pub fn cert_sha256(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        36, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_serial: optional string
  pub fn cert_serial(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        37, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        38, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        39, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_organization: optional string
  pub fn cert_organization(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        40, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_sans: repeated string
  pub fn cert_sans(self) -> ::protobuf::RepeatedView<'msg, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        41
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

  // cert_not_before: optional int64
  pub fn cert_not_before(self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        42, (0i64).into()
      ).try_into().unwrap()
    }
  }

  // cert_not_after: optional int64
  pub fn cert_not_after(self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        43, (0i64).into()
      ).try_into().unwrap()
    }
  }

  // cert_is_expired: optional bool
  pub fn cert_is_expired(self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        44, (false).into()
      ).try_into().unwrap()
    }
  }

  // cert_rpki_status: optional string
  pub fn cert_rpki_status(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        45, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cert_roa_prefix: optional string
  pub fn cert_roa_prefix(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        46, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // cves: repeated string
  pub fn cves(self) -> ::protobuf::RepeatedView<'msg, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        47
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

  // cvss_scores: repeated float
  pub fn cvss_scores(self) -> ::protobuf::RepeatedView<'msg, f32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        48
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<f32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

  // source_pop: optional string
  pub fn source_pop(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        49, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // event_id: optional string
  pub fn event_id(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        50, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `EnrichedHostView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for EnrichedHostView<'_> {}

// SAFETY:
// - `EnrichedHostView` is `Send` because while its alive a `EnrichedHostMut` cannot.
// - `EnrichedHostView` does not use thread-local data.
unsafe impl ::std::marker::Send for EnrichedHostView<'_> {}

impl<'msg> ::protobuf::AsView for EnrichedHostView<'msg> {
  type Proxied = EnrichedHost;
  fn as_view(&self) -> ::protobuf::View<'msg, EnrichedHost> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for EnrichedHostView<'msg> {
  fn into_view<'shorter>(self) -> EnrichedHostView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<EnrichedHost> for EnrichedHostView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> EnrichedHost {
    let mut dst = EnrichedHost::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<EnrichedHost> for EnrichedHostMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> EnrichedHost {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for EnrichedHost {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for EnrichedHostView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for EnrichedHostMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct EnrichedHostMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, EnrichedHost>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for EnrichedHostMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for EnrichedHostMut<'msg> {
  type Message = EnrichedHost;
}

impl ::std::fmt::Debug for EnrichedHostMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, EnrichedHost>> for EnrichedHostMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, EnrichedHost>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> EnrichedHostMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, EnrichedHost> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> EnrichedHost {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // ip_address: optional string
  pub fn ip_address(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ip_address(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // host_id: optional string
  pub fn host_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_host_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // asn: optional uint32
  pub fn asn(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_asn(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // organization: optional string
  pub fn organization(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        3, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_organization(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        3,
        val);
    }
  }

  // bgp_prefix: optional string
  pub fn bgp_prefix(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_bgp_prefix(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // bgp_as_path: optional string
  pub fn bgp_as_path(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_bgp_as_path(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        5,
        val);
    }
  }

  // reverse_dns: optional string
  pub fn reverse_dns(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_reverse_dns(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // forward_dns: optional string
  pub fn forward_dns(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_forward_dns(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // geo_tier_used: optional enum aegis.types.GeoTier
  pub fn geo_tier_used(&self) -> super::GeoTier {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        8, (super::GeoTier::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_tier_used(&mut self, val: super::GeoTier) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        8, val.into()
      )
    }
  }

  // geo_latitude: optional double
  pub fn geo_latitude(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        9, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_latitude(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        9, val.into()
      )
    }
  }

  // geo_longitude: optional double
  pub fn geo_longitude(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        10, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_longitude(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        10, val.into()
      )
    }
  }

  // geo_city: optional string
  pub fn geo_city(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_geo_city(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // geo_country: optional string
  pub fn geo_country(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_geo_country(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // geo_confidence: optional float
  pub fn geo_confidence(&self) -> f32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f32_at_index(
        13, (0f32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_confidence(&mut self, val: f32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f32_at_index(
        13, val.into()
      )
    }
  }

  // first_seen: optional int64
  pub fn first_seen(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        14, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_first_seen(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        14, val.into()
      )
    }
  }

  // last_seen: optional int64
  pub fn last_seen(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        15, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_last_seen(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        15, val.into()
      )
    }
  }

  // scan_count: optional int32
  pub fn scan_count(&self) -> i32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        16, (0i32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_scan_count(&mut self, val: i32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        16, val.into()
      )
    }
  }

  // status: optional enum aegis.types.HostStatus
  pub fn status(&self) -> super::HostStatus {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        17, (super::HostStatus::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_status(&mut self, val: super::HostStatus) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        17, val.into()
      )
    }
  }

  // port: optional uint32
  pub fn port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        18, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        18, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        19, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        19, val.into()
      )
    }
  }

  // reclassified_protocol: optional string
  pub fn reclassified_protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        20, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_reclassified_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        20,
        val);
    }
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(&self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        21, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_detection_method(&mut self, val: super::DetectionMethod) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        21, val.into()
      )
    }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        22, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sanitized_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        22,
        val);
    }
  }

  // banner_hash: optional string
  pub fn banner_hash(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        23, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_banner_hash(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        23,
        val);
    }
  }

  // ja4: optional string
  pub fn ja4(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        24, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        24,
        val);
    }
  }

  // ja4s: optional string
  pub fn ja4s(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        25, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4s(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        25,
        val);
    }
  }

  // tls_version: optional string
  pub fn tls_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        26, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        26,
        val);
    }
  }

  // cipher_suite: optional string
  pub fn cipher_suite(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        27, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cipher_suite(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        27,
        val);
    }
  }

  // sni: optional string
  pub fn sni(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        28, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sni(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        28,
        val);
    }
  }

  // alpn: optional string
  pub fn alpn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        29, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_alpn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        29,
        val);
    }
  }

  // http_method: optional string
  pub fn http_method(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        30, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_method(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        30,
        val);
    }
  }

  // http_status_code: optional uint32
  pub fn http_status_code(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        31, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_http_status_code(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        31, val.into()
      )
    }
  }

  // http_server: optional string
  pub fn http_server(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        32, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_server(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        32,
        val);
    }
  }

  // ssh_version: optional string
  pub fn ssh_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        33, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        33,
        val);
    }
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        34, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_key_exchange_algorithms(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        34,
        val);
    }
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        35, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_host_key_type(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        35,
        val);
    }
  }

  // cert_sha256: optional string
  pub fn cert_sha256(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        36, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_sha256(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        36,
        val);
    }
  }

  // cert_serial: optional string
  pub fn cert_serial(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        37, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_serial(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        37,
        val);
    }
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        38, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_subject_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        38,
        val);
    }
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        39, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_issuer_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        39,
        val);
    }
  }

  // cert_organization: optional string
  pub fn cert_organization(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        40, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_organization(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        40,
        val);
    }
  }

  // cert_sans: repeated string
  pub fn cert_sans(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        41
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cert_sans_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        41,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cert_sans(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        41,
        src);
    }
  }

  // cert_not_before: optional int64
  pub fn cert_not_before(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        42, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_not_before(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        42, val.into()
      )
    }
  }

  // cert_not_after: optional int64
  pub fn cert_not_after(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        43, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_not_after(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        43, val.into()
      )
    }
  }

  // cert_is_expired: optional bool
  pub fn cert_is_expired(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        44, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_is_expired(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        44, val.into()
      )
    }
  }

  // cert_rpki_status: optional string
  pub fn cert_rpki_status(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        45, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_rpki_status(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        45,
        val);
    }
  }

  // cert_roa_prefix: optional string
  pub fn cert_roa_prefix(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        46, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_roa_prefix(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        46,
        val);
    }
  }

  // cves: repeated string
  pub fn cves(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        47
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cves_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        47,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cves(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        47,
        src);
    }
  }

  // cvss_scores: repeated float
  pub fn cvss_scores(&self) -> ::protobuf::RepeatedView<'_, f32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        48
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<f32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cvss_scores_mut(&mut self) -> ::protobuf::RepeatedMut<'_, f32> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        48,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cvss_scores(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<f32>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        48,
        src);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        49, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        49,
        val);
    }
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        50, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        50,
        val);
    }
  }

}

// SAFETY:
// - `EnrichedHostMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for EnrichedHostMut<'_> {}

// SAFETY:
// - `EnrichedHostMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for EnrichedHostMut<'_> {}

impl<'msg> ::protobuf::AsView for EnrichedHostMut<'msg> {
  type Proxied = EnrichedHost;
  fn as_view(&self) -> ::protobuf::View<'_, EnrichedHost> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for EnrichedHostMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, EnrichedHost>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for EnrichedHostMut<'msg> {
  type MutProxied = EnrichedHost;
  fn as_mut(&mut self) -> EnrichedHostMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for EnrichedHostMut<'msg> {
  fn into_mut<'shorter>(self) -> EnrichedHostMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl EnrichedHost {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, EnrichedHost> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> EnrichedHostView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> EnrichedHostMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // ip_address: optional string
  pub fn ip_address(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ip_address(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // host_id: optional string
  pub fn host_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_host_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // asn: optional uint32
  pub fn asn(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_asn(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // organization: optional string
  pub fn organization(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        3, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_organization(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        3,
        val);
    }
  }

  // bgp_prefix: optional string
  pub fn bgp_prefix(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        4, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_bgp_prefix(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        4,
        val);
    }
  }

  // bgp_as_path: optional string
  pub fn bgp_as_path(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        5, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_bgp_as_path(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        5,
        val);
    }
  }

  // reverse_dns: optional string
  pub fn reverse_dns(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        6, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_reverse_dns(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        6,
        val);
    }
  }

  // forward_dns: optional string
  pub fn forward_dns(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        7, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_forward_dns(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        7,
        val);
    }
  }

  // geo_tier_used: optional enum aegis.types.GeoTier
  pub fn geo_tier_used(&self) -> super::GeoTier {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        8, (super::GeoTier::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_tier_used(&mut self, val: super::GeoTier) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        8, val.into()
      )
    }
  }

  // geo_latitude: optional double
  pub fn geo_latitude(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        9, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_latitude(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        9, val.into()
      )
    }
  }

  // geo_longitude: optional double
  pub fn geo_longitude(&self) -> f64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f64_at_index(
        10, (0f64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_longitude(&mut self, val: f64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f64_at_index(
        10, val.into()
      )
    }
  }

  // geo_city: optional string
  pub fn geo_city(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        11, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_geo_city(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        11,
        val);
    }
  }

  // geo_country: optional string
  pub fn geo_country(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        12, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_geo_country(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        12,
        val);
    }
  }

  // geo_confidence: optional float
  pub fn geo_confidence(&self) -> f32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_f32_at_index(
        13, (0f32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_geo_confidence(&mut self, val: f32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_f32_at_index(
        13, val.into()
      )
    }
  }

  // first_seen: optional int64
  pub fn first_seen(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        14, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_first_seen(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        14, val.into()
      )
    }
  }

  // last_seen: optional int64
  pub fn last_seen(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        15, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_last_seen(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        15, val.into()
      )
    }
  }

  // scan_count: optional int32
  pub fn scan_count(&self) -> i32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        16, (0i32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_scan_count(&mut self, val: i32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        16, val.into()
      )
    }
  }

  // status: optional enum aegis.types.HostStatus
  pub fn status(&self) -> super::HostStatus {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        17, (super::HostStatus::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_status(&mut self, val: super::HostStatus) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        17, val.into()
      )
    }
  }

  // port: optional uint32
  pub fn port(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        18, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_port(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        18, val.into()
      )
    }
  }

  // transport: optional enum aegis.types.Transport
  pub fn transport(&self) -> super::Transport {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        19, (super::Transport::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_transport(&mut self, val: super::Transport) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        19, val.into()
      )
    }
  }

  // reclassified_protocol: optional string
  pub fn reclassified_protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        20, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_reclassified_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        20,
        val);
    }
  }

  // detection_method: optional enum aegis.types.DetectionMethod
  pub fn detection_method(&self) -> super::DetectionMethod {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i32_at_index(
        21, (super::DetectionMethod::Unspecified).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_detection_method(&mut self, val: super::DetectionMethod) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i32_at_index(
        21, val.into()
      )
    }
  }

  // sanitized_banner: optional string
  pub fn sanitized_banner(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        22, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sanitized_banner(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        22,
        val);
    }
  }

  // banner_hash: optional string
  pub fn banner_hash(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        23, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_banner_hash(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        23,
        val);
    }
  }

  // ja4: optional string
  pub fn ja4(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        24, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        24,
        val);
    }
  }

  // ja4s: optional string
  pub fn ja4s(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        25, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ja4s(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        25,
        val);
    }
  }

  // tls_version: optional string
  pub fn tls_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        26, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        26,
        val);
    }
  }

  // cipher_suite: optional string
  pub fn cipher_suite(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        27, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cipher_suite(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        27,
        val);
    }
  }

  // sni: optional string
  pub fn sni(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        28, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_sni(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        28,
        val);
    }
  }

  // alpn: optional string
  pub fn alpn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        29, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_alpn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        29,
        val);
    }
  }

  // http_method: optional string
  pub fn http_method(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        30, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_method(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        30,
        val);
    }
  }

  // http_status_code: optional uint32
  pub fn http_status_code(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        31, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_http_status_code(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        31, val.into()
      )
    }
  }

  // http_server: optional string
  pub fn http_server(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        32, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_http_server(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        32,
        val);
    }
  }

  // ssh_version: optional string
  pub fn ssh_version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        33, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        33,
        val);
    }
  }

  // ssh_key_exchange_algorithms: optional string
  pub fn ssh_key_exchange_algorithms(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        34, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_key_exchange_algorithms(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        34,
        val);
    }
  }

  // ssh_host_key_type: optional string
  pub fn ssh_host_key_type(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        35, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_ssh_host_key_type(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        35,
        val);
    }
  }

  // cert_sha256: optional string
  pub fn cert_sha256(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        36, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_sha256(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        36,
        val);
    }
  }

  // cert_serial: optional string
  pub fn cert_serial(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        37, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_serial(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        37,
        val);
    }
  }

  // cert_subject_cn: optional string
  pub fn cert_subject_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        38, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_subject_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        38,
        val);
    }
  }

  // cert_issuer_cn: optional string
  pub fn cert_issuer_cn(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        39, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_issuer_cn(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        39,
        val);
    }
  }

  // cert_organization: optional string
  pub fn cert_organization(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        40, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_organization(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        40,
        val);
    }
  }

  // cert_sans: repeated string
  pub fn cert_sans(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        41
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cert_sans_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        41,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cert_sans(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        41,
        src);
    }
  }

  // cert_not_before: optional int64
  pub fn cert_not_before(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        42, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_not_before(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        42, val.into()
      )
    }
  }

  // cert_not_after: optional int64
  pub fn cert_not_after(&self) -> i64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_i64_at_index(
        43, (0i64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_not_after(&mut self, val: i64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_i64_at_index(
        43, val.into()
      )
    }
  }

  // cert_is_expired: optional bool
  pub fn cert_is_expired(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        44, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_cert_is_expired(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        44, val.into()
      )
    }
  }

  // cert_rpki_status: optional string
  pub fn cert_rpki_status(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        45, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_rpki_status(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        45,
        val);
    }
  }

  // cert_roa_prefix: optional string
  pub fn cert_roa_prefix(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        46, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_cert_roa_prefix(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        46,
        val);
    }
  }

  // cves: repeated string
  pub fn cves(&self) -> ::protobuf::RepeatedView<'_, ::protobuf::ProtoString> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        47
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<::protobuf::ProtoString>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cves_mut(&mut self) -> ::protobuf::RepeatedMut<'_, ::protobuf::ProtoString> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        47,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cves(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<::protobuf::ProtoString>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        47,
        src);
    }
  }

  // cvss_scores: repeated float
  pub fn cvss_scores(&self) -> ::protobuf::RepeatedView<'_, f32> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        48
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<f32>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn cvss_scores_mut(&mut self) -> ::protobuf::RepeatedMut<'_, f32> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        48,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_cvss_scores(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<f32>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        48,
        src);
    }
  }

  // source_pop: optional string
  pub fn source_pop(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        49, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_source_pop(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        49,
        val);
    }
  }

  // event_id: optional string
  pub fn event_id(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        50, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_event_id(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        50,
        val);
    }
  }

}  // impl EnrichedHost

impl ::std::ops::Drop for EnrichedHost {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for EnrichedHost {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for EnrichedHost {
  type Proxied = Self;
  fn as_view(&self) -> EnrichedHostView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for EnrichedHost {
  type MutProxied = Self;
  fn as_mut(&mut self) -> EnrichedHostMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for EnrichedHost {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__EnrichedHost_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$N1X1X)P1X1X1X1X1X.P P P1X1X!P+P+P(P.Pa)P.P1X.P1X1Xd1X1X1X1X1X1Xd1X)P1X1X1X1Xd1X1X1X1X1XET+P+P/P1X1XiET7h1X1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__EnrichedHost_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__EnrichedHost_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for EnrichedHost {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for EnrichedHost {
  type Msg = EnrichedHost;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<EnrichedHost> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for EnrichedHost {
  type Msg = EnrichedHost;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<EnrichedHost> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for EnrichedHostMut<'_> {
  type Msg = EnrichedHost;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<EnrichedHost> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for EnrichedHostMut<'_> {
  type Msg = EnrichedHost;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<EnrichedHost> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for EnrichedHostView<'_> {
  type Msg = EnrichedHost;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<EnrichedHost> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for EnrichedHostMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__ClassifyRequest_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct ClassifyRequest {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<ClassifyRequest>
}

impl ::protobuf::Message for ClassifyRequest {
  type MessageView<'msg> = ClassifyRequestView<'msg>;
  type MessageMut<'msg> = ClassifyRequestMut<'msg>;
}

impl ::std::default::Default for ClassifyRequest {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for ClassifyRequest {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `ClassifyRequest` is `Sync` because it does not implement interior mutability.
//    Neither does `ClassifyRequestMut`.
unsafe impl ::std::marker::Sync for ClassifyRequest {}

// SAFETY:
// - `ClassifyRequest` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyRequest {}

impl ::protobuf::Proxied for ClassifyRequest {
  type View<'msg> = ClassifyRequestView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for ClassifyRequest {}

impl ::protobuf::MutProxied for ClassifyRequest {
  type Mut<'msg> = ClassifyRequestMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ClassifyRequestView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyRequest>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyRequestView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for ClassifyRequestView<'msg> {
  type Message = ClassifyRequest;
}

impl ::std::fmt::Debug for ClassifyRequestView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for ClassifyRequestView<'_> {
  fn default() -> ClassifyRequestView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyRequest>> for ClassifyRequestView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyRequest>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyRequestView<'msg> {

  pub fn to_owned(&self) -> ClassifyRequest {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // payload: optional bytes
  pub fn payload(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

  // tls_hint: optional string
  pub fn tls_hint(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // source_ip: optional bytes
  pub fn source_ip(self) -> ::protobuf::View<'msg, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }

}

// SAFETY:
// - `ClassifyRequestView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for ClassifyRequestView<'_> {}

// SAFETY:
// - `ClassifyRequestView` is `Send` because while its alive a `ClassifyRequestMut` cannot.
// - `ClassifyRequestView` does not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyRequestView<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyRequestView<'msg> {
  type Proxied = ClassifyRequest;
  fn as_view(&self) -> ::protobuf::View<'msg, ClassifyRequest> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyRequestView<'msg> {
  fn into_view<'shorter>(self) -> ClassifyRequestView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyRequest> for ClassifyRequestView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyRequest {
    let mut dst = ClassifyRequest::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyRequest> for ClassifyRequestMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyRequest {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for ClassifyRequest {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyRequestView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyRequestMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct ClassifyRequestMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyRequest>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyRequestMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for ClassifyRequestMut<'msg> {
  type Message = ClassifyRequest;
}

impl ::std::fmt::Debug for ClassifyRequestMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyRequest>> for ClassifyRequestMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyRequest>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyRequestMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyRequest> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> ClassifyRequest {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // payload: optional bytes
  pub fn payload(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_payload(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // tls_hint: optional string
  pub fn tls_hint(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_hint(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // source_ip: optional bytes
  pub fn source_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_source_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        2,
        val);
    }
  }

}

// SAFETY:
// - `ClassifyRequestMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for ClassifyRequestMut<'_> {}

// SAFETY:
// - `ClassifyRequestMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for ClassifyRequestMut<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyRequestMut<'msg> {
  type Proxied = ClassifyRequest;
  fn as_view(&self) -> ::protobuf::View<'_, ClassifyRequest> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyRequestMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, ClassifyRequest>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for ClassifyRequestMut<'msg> {
  type MutProxied = ClassifyRequest;
  fn as_mut(&mut self) -> ClassifyRequestMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for ClassifyRequestMut<'msg> {
  fn into_mut<'shorter>(self) -> ClassifyRequestMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl ClassifyRequest {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, ClassifyRequest> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> ClassifyRequestView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> ClassifyRequestMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // payload: optional bytes
  pub fn payload(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_payload(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // tls_hint: optional string
  pub fn tls_hint(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_tls_hint(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // source_ip: optional bytes
  pub fn source_ip(&self) -> ::protobuf::View<'_, ::protobuf::ProtoBytes> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    unsafe { str_view.as_ref() }
  }
  pub fn set_source_ip(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoBytes>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_bytes_field(
        ::protobuf::AsMut::as_mut(self).inner,
        2,
        val);
    }
  }

}  // impl ClassifyRequest

impl ::std::ops::Drop for ClassifyRequest {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for ClassifyRequest {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for ClassifyRequest {
  type Proxied = Self;
  fn as_view(&self) -> ClassifyRequestView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for ClassifyRequest {
  type MutProxied = Self;
  fn as_mut(&mut self) -> ClassifyRequestMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for ClassifyRequest {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__ClassifyRequest_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$0P1X0P");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__ClassifyRequest_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__ClassifyRequest_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyRequest {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyRequest {
  type Msg = ClassifyRequest;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyRequest> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyRequest {
  type Msg = ClassifyRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyRequest> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyRequestMut<'_> {
  type Msg = ClassifyRequest;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyRequest> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyRequestMut<'_> {
  type Msg = ClassifyRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyRequest> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyRequestView<'_> {
  type Msg = ClassifyRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyRequest> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyRequestMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__ClassifyResponse_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct ClassifyResponse {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<ClassifyResponse>
}

impl ::protobuf::Message for ClassifyResponse {
  type MessageView<'msg> = ClassifyResponseView<'msg>;
  type MessageMut<'msg> = ClassifyResponseMut<'msg>;
}

impl ::std::default::Default for ClassifyResponse {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for ClassifyResponse {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `ClassifyResponse` is `Sync` because it does not implement interior mutability.
//    Neither does `ClassifyResponseMut`.
unsafe impl ::std::marker::Sync for ClassifyResponse {}

// SAFETY:
// - `ClassifyResponse` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyResponse {}

impl ::protobuf::Proxied for ClassifyResponse {
  type View<'msg> = ClassifyResponseView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for ClassifyResponse {}

impl ::protobuf::MutProxied for ClassifyResponse {
  type Mut<'msg> = ClassifyResponseMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ClassifyResponseView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyResponseView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for ClassifyResponseView<'msg> {
  type Message = ClassifyResponse;
}

impl ::std::fmt::Debug for ClassifyResponseView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for ClassifyResponseView<'_> {
  fn default() -> ClassifyResponseView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyResponse>> for ClassifyResponseView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyResponseView<'msg> {

  pub fn to_owned(&self) -> ClassifyResponse {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // protocol: optional string
  pub fn protocol(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // category: optional string
  pub fn category(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

  // priority: optional uint32
  pub fn priority(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // matched: optional bool
  pub fn matched(self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        3, (false).into()
      ).try_into().unwrap()
    }
  }

  // latency_us: optional uint64
  pub fn latency_us(self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        4, (0u64).into()
      ).try_into().unwrap()
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }

}

// SAFETY:
// - `ClassifyResponseView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for ClassifyResponseView<'_> {}

// SAFETY:
// - `ClassifyResponseView` is `Send` because while its alive a `ClassifyResponseMut` cannot.
// - `ClassifyResponseView` does not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyResponseView<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyResponseView<'msg> {
  type Proxied = ClassifyResponse;
  fn as_view(&self) -> ::protobuf::View<'msg, ClassifyResponse> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyResponseView<'msg> {
  fn into_view<'shorter>(self) -> ClassifyResponseView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyResponse> for ClassifyResponseView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyResponse {
    let mut dst = ClassifyResponse::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyResponse> for ClassifyResponseMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyResponse {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for ClassifyResponse {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyResponseView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyResponseMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct ClassifyResponseMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyResponseMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for ClassifyResponseMut<'msg> {
  type Message = ClassifyResponse;
}

impl ::std::fmt::Debug for ClassifyResponseMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyResponse>> for ClassifyResponseMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyResponseMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyResponse> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> ClassifyResponse {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // protocol: optional string
  pub fn protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // category: optional string
  pub fn category(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_category(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // priority: optional uint32
  pub fn priority(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_priority(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // matched: optional bool
  pub fn matched(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        3, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_matched(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        3, val.into()
      )
    }
  }

  // latency_us: optional uint64
  pub fn latency_us(&self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        4, (0u64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_latency_us(&mut self, val: u64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u64_at_index(
        4, val.into()
      )
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_signature_count(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        5, val.into()
      )
    }
  }

}

// SAFETY:
// - `ClassifyResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for ClassifyResponseMut<'_> {}

// SAFETY:
// - `ClassifyResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for ClassifyResponseMut<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyResponseMut<'msg> {
  type Proxied = ClassifyResponse;
  fn as_view(&self) -> ::protobuf::View<'_, ClassifyResponse> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyResponseMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, ClassifyResponse>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for ClassifyResponseMut<'msg> {
  type MutProxied = ClassifyResponse;
  fn as_mut(&mut self) -> ClassifyResponseMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for ClassifyResponseMut<'msg> {
  fn into_mut<'shorter>(self) -> ClassifyResponseMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl ClassifyResponse {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, ClassifyResponse> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> ClassifyResponseView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> ClassifyResponseMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // protocol: optional string
  pub fn protocol(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        0, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_protocol(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        val);
    }
  }

  // category: optional string
  pub fn category(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        1, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_category(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        1,
        val);
    }
  }

  // priority: optional uint32
  pub fn priority(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        2, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_priority(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        2, val.into()
      )
    }
  }

  // matched: optional bool
  pub fn matched(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        3, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_matched(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        3, val.into()
      )
    }
  }

  // latency_us: optional uint64
  pub fn latency_us(&self) -> u64 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u64_at_index(
        4, (0u64).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_latency_us(&mut self, val: u64) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u64_at_index(
        4, val.into()
      )
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        5, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_signature_count(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        5, val.into()
      )
    }
  }

}  // impl ClassifyResponse

impl ::std::ops::Drop for ClassifyResponse {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for ClassifyResponse {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for ClassifyResponse {
  type Proxied = Self;
  fn as_view(&self) -> ClassifyResponseView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for ClassifyResponse {
  type MutProxied = Self;
  fn as_mut(&mut self) -> ClassifyResponseMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for ClassifyResponse {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__ClassifyResponse_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$1X1X)P/P,P)P");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__ClassifyResponse_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__ClassifyResponse_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyResponse {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyResponse {
  type Msg = ClassifyResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyResponse {
  type Msg = ClassifyResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyResponseMut<'_> {
  type Msg = ClassifyResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyResponseMut<'_> {
  type Msg = ClassifyResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyResponseView<'_> {
  type Msg = ClassifyResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyResponse> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyResponseMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__ClassifyBatchRequest_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct ClassifyBatchRequest {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<ClassifyBatchRequest>
}

impl ::protobuf::Message for ClassifyBatchRequest {
  type MessageView<'msg> = ClassifyBatchRequestView<'msg>;
  type MessageMut<'msg> = ClassifyBatchRequestMut<'msg>;
}

impl ::std::default::Default for ClassifyBatchRequest {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for ClassifyBatchRequest {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `ClassifyBatchRequest` is `Sync` because it does not implement interior mutability.
//    Neither does `ClassifyBatchRequestMut`.
unsafe impl ::std::marker::Sync for ClassifyBatchRequest {}

// SAFETY:
// - `ClassifyBatchRequest` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyBatchRequest {}

impl ::protobuf::Proxied for ClassifyBatchRequest {
  type View<'msg> = ClassifyBatchRequestView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for ClassifyBatchRequest {}

impl ::protobuf::MutProxied for ClassifyBatchRequest {
  type Mut<'msg> = ClassifyBatchRequestMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ClassifyBatchRequestView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchRequest>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyBatchRequestView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for ClassifyBatchRequestView<'msg> {
  type Message = ClassifyBatchRequest;
}

impl ::std::fmt::Debug for ClassifyBatchRequestView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for ClassifyBatchRequestView<'_> {
  fn default() -> ClassifyBatchRequestView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchRequest>> for ClassifyBatchRequestView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchRequest>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyBatchRequestView<'msg> {

  pub fn to_owned(&self) -> ClassifyBatchRequest {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // requests: repeated message aegis.types.ClassifyRequest
  pub fn requests(self) -> ::protobuf::RepeatedView<'msg, super::ClassifyRequest> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyRequest>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

}

// SAFETY:
// - `ClassifyBatchRequestView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for ClassifyBatchRequestView<'_> {}

// SAFETY:
// - `ClassifyBatchRequestView` is `Send` because while its alive a `ClassifyBatchRequestMut` cannot.
// - `ClassifyBatchRequestView` does not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyBatchRequestView<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyBatchRequestView<'msg> {
  type Proxied = ClassifyBatchRequest;
  fn as_view(&self) -> ::protobuf::View<'msg, ClassifyBatchRequest> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyBatchRequestView<'msg> {
  fn into_view<'shorter>(self) -> ClassifyBatchRequestView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyBatchRequest> for ClassifyBatchRequestView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyBatchRequest {
    let mut dst = ClassifyBatchRequest::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyBatchRequest> for ClassifyBatchRequestMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyBatchRequest {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for ClassifyBatchRequest {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyBatchRequestView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyBatchRequestMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct ClassifyBatchRequestMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchRequest>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyBatchRequestMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for ClassifyBatchRequestMut<'msg> {
  type Message = ClassifyBatchRequest;
}

impl ::std::fmt::Debug for ClassifyBatchRequestMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchRequest>> for ClassifyBatchRequestMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchRequest>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyBatchRequestMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchRequest> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> ClassifyBatchRequest {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // requests: repeated message aegis.types.ClassifyRequest
  pub fn requests(&self) -> ::protobuf::RepeatedView<'_, super::ClassifyRequest> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyRequest>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn requests_mut(&mut self) -> ::protobuf::RepeatedMut<'_, super::ClassifyRequest> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        0,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_requests(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<super::ClassifyRequest>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        src);
    }
  }

}

// SAFETY:
// - `ClassifyBatchRequestMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for ClassifyBatchRequestMut<'_> {}

// SAFETY:
// - `ClassifyBatchRequestMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for ClassifyBatchRequestMut<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyBatchRequestMut<'msg> {
  type Proxied = ClassifyBatchRequest;
  fn as_view(&self) -> ::protobuf::View<'_, ClassifyBatchRequest> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyBatchRequestMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, ClassifyBatchRequest>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for ClassifyBatchRequestMut<'msg> {
  type MutProxied = ClassifyBatchRequest;
  fn as_mut(&mut self) -> ClassifyBatchRequestMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for ClassifyBatchRequestMut<'msg> {
  fn into_mut<'shorter>(self) -> ClassifyBatchRequestMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl ClassifyBatchRequest {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, ClassifyBatchRequest> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> ClassifyBatchRequestView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> ClassifyBatchRequestMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // requests: repeated message aegis.types.ClassifyRequest
  pub fn requests(&self) -> ::protobuf::RepeatedView<'_, super::ClassifyRequest> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyRequest>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn requests_mut(&mut self) -> ::protobuf::RepeatedMut<'_, super::ClassifyRequest> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        0,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_requests(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<super::ClassifyRequest>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        src);
    }
  }

}  // impl ClassifyBatchRequest

impl ::std::ops::Drop for ClassifyBatchRequest {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for ClassifyBatchRequest {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for ClassifyBatchRequest {
  type Proxied = Self;
  fn as_view(&self) -> ClassifyBatchRequestView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for ClassifyBatchRequest {
  type MutProxied = Self;
  fn as_mut(&mut self) -> ClassifyBatchRequestMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for ClassifyBatchRequest {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__ClassifyBatchRequest_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$G");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__ClassifyBatchRequest_msg_init.0, &[<super::ClassifyRequest as ::protobuf::__internal::runtime::AssociatedMiniTable>::mini_table(),
            ], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__ClassifyBatchRequest_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyBatchRequest {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyBatchRequest {
  type Msg = ClassifyBatchRequest;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchRequest> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchRequest {
  type Msg = ClassifyBatchRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchRequest> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyBatchRequestMut<'_> {
  type Msg = ClassifyBatchRequest;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchRequest> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchRequestMut<'_> {
  type Msg = ClassifyBatchRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchRequest> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchRequestView<'_> {
  type Msg = ClassifyBatchRequest;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchRequest> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyBatchRequestMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__ClassifyBatchResponse_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct ClassifyBatchResponse {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<ClassifyBatchResponse>
}

impl ::protobuf::Message for ClassifyBatchResponse {
  type MessageView<'msg> = ClassifyBatchResponseView<'msg>;
  type MessageMut<'msg> = ClassifyBatchResponseMut<'msg>;
}

impl ::std::default::Default for ClassifyBatchResponse {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for ClassifyBatchResponse {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `ClassifyBatchResponse` is `Sync` because it does not implement interior mutability.
//    Neither does `ClassifyBatchResponseMut`.
unsafe impl ::std::marker::Sync for ClassifyBatchResponse {}

// SAFETY:
// - `ClassifyBatchResponse` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyBatchResponse {}

impl ::protobuf::Proxied for ClassifyBatchResponse {
  type View<'msg> = ClassifyBatchResponseView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for ClassifyBatchResponse {}

impl ::protobuf::MutProxied for ClassifyBatchResponse {
  type Mut<'msg> = ClassifyBatchResponseMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct ClassifyBatchResponseView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyBatchResponseView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for ClassifyBatchResponseView<'msg> {
  type Message = ClassifyBatchResponse;
}

impl ::std::fmt::Debug for ClassifyBatchResponseView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for ClassifyBatchResponseView<'_> {
  fn default() -> ClassifyBatchResponseView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchResponse>> for ClassifyBatchResponseView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, ClassifyBatchResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyBatchResponseView<'msg> {

  pub fn to_owned(&self) -> ClassifyBatchResponse {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // responses: repeated message aegis.types.ClassifyResponse
  pub fn responses(self) -> ::protobuf::RepeatedView<'msg, super::ClassifyResponse> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyResponse>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }

}

// SAFETY:
// - `ClassifyBatchResponseView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for ClassifyBatchResponseView<'_> {}

// SAFETY:
// - `ClassifyBatchResponseView` is `Send` because while its alive a `ClassifyBatchResponseMut` cannot.
// - `ClassifyBatchResponseView` does not use thread-local data.
unsafe impl ::std::marker::Send for ClassifyBatchResponseView<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyBatchResponseView<'msg> {
  type Proxied = ClassifyBatchResponse;
  fn as_view(&self) -> ::protobuf::View<'msg, ClassifyBatchResponse> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyBatchResponseView<'msg> {
  fn into_view<'shorter>(self) -> ClassifyBatchResponseView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyBatchResponse> for ClassifyBatchResponseView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyBatchResponse {
    let mut dst = ClassifyBatchResponse::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<ClassifyBatchResponse> for ClassifyBatchResponseMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> ClassifyBatchResponse {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for ClassifyBatchResponse {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyBatchResponseView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for ClassifyBatchResponseMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct ClassifyBatchResponseMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for ClassifyBatchResponseMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for ClassifyBatchResponseMut<'msg> {
  type Message = ClassifyBatchResponse;
}

impl ::std::fmt::Debug for ClassifyBatchResponseMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchResponse>> for ClassifyBatchResponseMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> ClassifyBatchResponseMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, ClassifyBatchResponse> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> ClassifyBatchResponse {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // responses: repeated message aegis.types.ClassifyResponse
  pub fn responses(&self) -> ::protobuf::RepeatedView<'_, super::ClassifyResponse> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyResponse>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn responses_mut(&mut self) -> ::protobuf::RepeatedMut<'_, super::ClassifyResponse> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        0,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_responses(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<super::ClassifyResponse>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        src);
    }
  }

}

// SAFETY:
// - `ClassifyBatchResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for ClassifyBatchResponseMut<'_> {}

// SAFETY:
// - `ClassifyBatchResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for ClassifyBatchResponseMut<'_> {}

impl<'msg> ::protobuf::AsView for ClassifyBatchResponseMut<'msg> {
  type Proxied = ClassifyBatchResponse;
  fn as_view(&self) -> ::protobuf::View<'_, ClassifyBatchResponse> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for ClassifyBatchResponseMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, ClassifyBatchResponse>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for ClassifyBatchResponseMut<'msg> {
  type MutProxied = ClassifyBatchResponse;
  fn as_mut(&mut self) -> ClassifyBatchResponseMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for ClassifyBatchResponseMut<'msg> {
  fn into_mut<'shorter>(self) -> ClassifyBatchResponseMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl ClassifyBatchResponse {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, ClassifyBatchResponse> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> ClassifyBatchResponseView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> ClassifyBatchResponseMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // responses: repeated message aegis.types.ClassifyResponse
  pub fn responses(&self) -> ::protobuf::RepeatedView<'_, super::ClassifyResponse> {
    unsafe {
      self.inner.ptr().get_array_at_index(
        0
      )
    }.map_or_else(
        ::protobuf::__internal::runtime::empty_array::<super::ClassifyResponse>,
        |raw| unsafe {
          ::protobuf::RepeatedView::from_raw(::protobuf::__internal::Private, raw)
        }
      )
  }
  pub fn responses_mut(&mut self) -> ::protobuf::RepeatedMut<'_, super::ClassifyResponse> {
    unsafe {
      let raw_array = self.inner.ptr_mut().get_or_create_mutable_array_at_index(
        0,
        self.inner.arena()
      ).expect("alloc should not fail");
      ::protobuf::RepeatedMut::from_inner(
        ::protobuf::__internal::Private,
        ::protobuf::__internal::runtime::InnerRepeatedMut::new(
          raw_array, self.inner.arena(),
        ),
      )
    }
  }
  pub fn set_responses(&mut self, src: impl ::protobuf::IntoProxied<::protobuf::Repeated<super::ClassifyResponse>>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_repeated_field(
        ::protobuf::AsMut::as_mut(self).inner,
        0,
        src);
    }
  }

}  // impl ClassifyBatchResponse

impl ::std::ops::Drop for ClassifyBatchResponse {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for ClassifyBatchResponse {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for ClassifyBatchResponse {
  type Proxied = Self;
  fn as_view(&self) -> ClassifyBatchResponseView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for ClassifyBatchResponse {
  type MutProxied = Self;
  fn as_mut(&mut self) -> ClassifyBatchResponseMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for ClassifyBatchResponse {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__ClassifyBatchResponse_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$G");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__ClassifyBatchResponse_msg_init.0, &[<super::ClassifyResponse as ::protobuf::__internal::runtime::AssociatedMiniTable>::mini_table(),
            ], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__ClassifyBatchResponse_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyBatchResponse {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyBatchResponse {
  type Msg = ClassifyBatchResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchResponse {
  type Msg = ClassifyBatchResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for ClassifyBatchResponseMut<'_> {
  type Msg = ClassifyBatchResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchResponseMut<'_> {
  type Msg = ClassifyBatchResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for ClassifyBatchResponseView<'_> {
  type Msg = ClassifyBatchResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<ClassifyBatchResponse> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for ClassifyBatchResponseMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



// This variable must not be referenced except by protobuf generated
// code.
pub(crate) static mut aegis__types__HealthCheckResponse_msg_init: ::protobuf::__internal::runtime::MiniTableInitPtr =
    ::protobuf::__internal::runtime::MiniTableInitPtr(::protobuf::__internal::runtime::MiniTablePtr::dangling());
#[allow(non_camel_case_types)]
pub struct HealthCheckResponse {
  inner: ::protobuf::__internal::runtime::OwnedMessageInner<HealthCheckResponse>
}

impl ::protobuf::Message for HealthCheckResponse {
  type MessageView<'msg> = HealthCheckResponseView<'msg>;
  type MessageMut<'msg> = HealthCheckResponseMut<'msg>;
}

impl ::std::default::Default for HealthCheckResponse {
  fn default() -> Self {
    Self::new()
  }
}

impl ::std::fmt::Debug for HealthCheckResponse {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

// SAFETY:
// - `HealthCheckResponse` is `Sync` because it does not implement interior mutability.
//    Neither does `HealthCheckResponseMut`.
unsafe impl ::std::marker::Sync for HealthCheckResponse {}

// SAFETY:
// - `HealthCheckResponse` is `Send` because it uniquely owns its arena and does
//   not use thread-local data.
unsafe impl ::std::marker::Send for HealthCheckResponse {}

impl ::protobuf::Proxied for HealthCheckResponse {
  type View<'msg> = HealthCheckResponseView<'msg>;
}

impl ::protobuf::__internal::SealedInternal for HealthCheckResponse {}

impl ::protobuf::MutProxied for HealthCheckResponse {
  type Mut<'msg> = HealthCheckResponseMut<'msg>;
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HealthCheckResponseView<'msg> {
  inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, HealthCheckResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for HealthCheckResponseView<'msg> {}

impl<'msg> ::protobuf::MessageView<'msg> for HealthCheckResponseView<'msg> {
  type Message = HealthCheckResponse;
}

impl ::std::fmt::Debug for HealthCheckResponseView<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl ::std::default::Default for HealthCheckResponseView<'_> {
  fn default() -> HealthCheckResponseView<'static> {
    ::protobuf::__internal::runtime::MessageViewInner::default().into()
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageViewInner<'msg, HealthCheckResponse>> for HealthCheckResponseView<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageViewInner<'msg, HealthCheckResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> HealthCheckResponseView<'msg> {

  pub fn to_owned(&self) -> HealthCheckResponse {
    ::protobuf::IntoProxied::into_proxied(*self, ::protobuf::__internal::Private)
  }

  // ready: optional bool
  pub fn ready(self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        0, (false).into()
      ).try_into().unwrap()
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        1, (0u32).into()
      ).try_into().unwrap()
    }
  }

  // version: optional string
  pub fn version(self) -> ::protobuf::View<'msg, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }

}

// SAFETY:
// - `HealthCheckResponseView` is `Sync` because it does not support mutation.
unsafe impl ::std::marker::Sync for HealthCheckResponseView<'_> {}

// SAFETY:
// - `HealthCheckResponseView` is `Send` because while its alive a `HealthCheckResponseMut` cannot.
// - `HealthCheckResponseView` does not use thread-local data.
unsafe impl ::std::marker::Send for HealthCheckResponseView<'_> {}

impl<'msg> ::protobuf::AsView for HealthCheckResponseView<'msg> {
  type Proxied = HealthCheckResponse;
  fn as_view(&self) -> ::protobuf::View<'msg, HealthCheckResponse> {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for HealthCheckResponseView<'msg> {
  fn into_view<'shorter>(self) -> HealthCheckResponseView<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

impl<'msg> ::protobuf::IntoProxied<HealthCheckResponse> for HealthCheckResponseView<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> HealthCheckResponse {
    let mut dst = HealthCheckResponse::new();
    assert!(unsafe {
      dst.inner.ptr_mut().deep_copy(self.inner.ptr(), dst.inner.arena())
    });
    dst
  }
}

impl<'msg> ::protobuf::IntoProxied<HealthCheckResponse> for HealthCheckResponseMut<'msg> {
  fn into_proxied(self, _private: ::protobuf::__internal::Private) -> HealthCheckResponse {
    ::protobuf::IntoProxied::into_proxied(::protobuf::IntoView::into_view(self), _private)
  }
}

impl ::protobuf::__internal::EntityType for HealthCheckResponse {
    type Tag = ::protobuf::__internal::entity_tag::MessageTag;
}

impl<'msg> ::protobuf::__internal::EntityType for HealthCheckResponseView<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::ViewProxyTag;
}

impl<'msg> ::protobuf::__internal::EntityType for HealthCheckResponseMut<'msg> {
    type Tag = ::protobuf::__internal::entity_tag::MutProxyTag;
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub struct HealthCheckResponseMut<'msg> {
  inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, HealthCheckResponse>,
}

impl<'msg> ::protobuf::__internal::SealedInternal for HealthCheckResponseMut<'msg> {}

impl<'msg> ::protobuf::MessageMut<'msg> for HealthCheckResponseMut<'msg> {
  type Message = HealthCheckResponse;
}

impl ::std::fmt::Debug for HealthCheckResponseMut<'_> {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    write!(f, "{}", ::protobuf::__internal::runtime::debug_string(self))
  }
}

impl<'msg> From<::protobuf::__internal::runtime::MessageMutInner<'msg, HealthCheckResponse>> for HealthCheckResponseMut<'msg> {
  fn from(inner: ::protobuf::__internal::runtime::MessageMutInner<'msg, HealthCheckResponse>) -> Self {
    Self { inner }
  }
}

#[allow(dead_code)]
impl<'msg> HealthCheckResponseMut<'msg> {

  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private)
    -> ::protobuf::__internal::runtime::MessageMutInner<'msg, HealthCheckResponse> {
    self.inner.reborrow()
  }

  pub fn to_owned(&self) -> HealthCheckResponse {
    ::protobuf::AsView::as_view(self).to_owned()
  }

  // ready: optional bool
  pub fn ready(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        0, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_ready(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        0, val.into()
      )
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        1, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_signature_count(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        1, val.into()
      )
    }
  }

  // version: optional string
  pub fn version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        2,
        val);
    }
  }

}

// SAFETY:
// - `HealthCheckResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Send for HealthCheckResponseMut<'_> {}

// SAFETY:
// - `HealthCheckResponseMut` does not perform any shared mutation.
unsafe impl ::std::marker::Sync for HealthCheckResponseMut<'_> {}

impl<'msg> ::protobuf::AsView for HealthCheckResponseMut<'msg> {
  type Proxied = HealthCheckResponse;
  fn as_view(&self) -> ::protobuf::View<'_, HealthCheckResponse> {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for HealthCheckResponseMut<'msg> {
  fn into_view<'shorter>(self) -> ::protobuf::View<'shorter, HealthCheckResponse>
  where
      'msg: 'shorter {
    self.inner.as_view().into()
  }
}

impl<'msg> ::protobuf::AsMut for HealthCheckResponseMut<'msg> {
  type MutProxied = HealthCheckResponse;
  fn as_mut(&mut self) -> HealthCheckResponseMut<'msg> {
    self.inner.reborrow().into()
  }
}

impl<'msg> ::protobuf::IntoMut<'msg> for HealthCheckResponseMut<'msg> {
  fn into_mut<'shorter>(self) -> HealthCheckResponseMut<'shorter>
  where
      'msg: 'shorter {
    self
  }
}

#[allow(dead_code)]
impl HealthCheckResponse {
  pub fn new() -> Self {
    Self { inner: ::protobuf::__internal::runtime::OwnedMessageInner::<Self>::new() }
  }


  #[doc(hidden)]
  pub fn as_message_mut_inner(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessageMutInner<'_, HealthCheckResponse> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner)
  }

  pub fn as_view(&self) -> HealthCheckResponseView<'_> {
    ::protobuf::__internal::runtime::MessageViewInner::view_of_owned(&self.inner).into()
  }

  pub fn as_mut(&mut self) -> HealthCheckResponseMut<'_> {
    ::protobuf::__internal::runtime::MessageMutInner::mut_of_owned(&mut self.inner).into()
  }

  // ready: optional bool
  pub fn ready(&self) -> bool {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_bool_at_index(
        0, (false).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_ready(&mut self, val: bool) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_bool_at_index(
        0, val.into()
      )
    }
  }

  // signature_count: optional uint32
  pub fn signature_count(&self) -> u32 {
    unsafe {
      // TODO: b/361751487: This .into() and .try_into() is only
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      // perfectly (and do an unchecked conversion for
      // i32->enum types, since even for closed enums we trust
      // upb to only return one of the named values).
      self.inner.ptr().get_u32_at_index(
        1, (0u32).into()
      ).try_into().unwrap()
    }
  }
  pub fn set_signature_count(&mut self, val: u32) {
    unsafe {
      // TODO: b/361751487: This .into() is only here
      // here for the enum<->i32 case, we should avoid it for
      // other primitives where the types naturally match
      //perfectly.
      self.inner.ptr_mut().set_base_field_u32_at_index(
        1, val.into()
      )
    }
  }

  // version: optional string
  pub fn version(&self) -> ::protobuf::View<'_, ::protobuf::ProtoString> {
    let str_view = unsafe {
      self.inner.ptr().get_string_at_index(
        2, (b"").into()
      )
    };
    ::protobuf::ProtoStr::from_utf8_unchecked(unsafe { str_view.as_ref() })
  }
  pub fn set_version(&mut self, val: impl ::protobuf::IntoProxied<::protobuf::ProtoString>) {
    unsafe {
      ::protobuf::__internal::runtime::message_set_string_field(
        ::protobuf::AsMut::as_mut(self).inner,
        2,
        val);
    }
  }

}  // impl HealthCheckResponse

impl ::std::ops::Drop for HealthCheckResponse {
  #[inline]
  fn drop(&mut self) {
  }
}

impl ::std::clone::Clone for HealthCheckResponse {
  fn clone(&self) -> Self {
    self.as_view().to_owned()
  }
}

impl ::protobuf::AsView for HealthCheckResponse {
  type Proxied = Self;
  fn as_view(&self) -> HealthCheckResponseView<'_> {
    self.as_view()
  }
}

impl ::protobuf::AsMut for HealthCheckResponse {
  type MutProxied = Self;
  fn as_mut(&mut self) -> HealthCheckResponseMut<'_> {
    self.as_mut()
  }
}

unsafe impl ::protobuf::__internal::runtime::AssociatedMiniTable for HealthCheckResponse {
  fn mini_table() -> ::protobuf::__internal::runtime::MiniTablePtr {
    static ONCE_LOCK: ::std::sync::OnceLock<::protobuf::__internal::runtime::MiniTableInitPtr> =
        ::std::sync::OnceLock::new();
    unsafe {
      ONCE_LOCK.get_or_init(|| {
        super::aegis__types__HealthCheckResponse_msg_init.0 =
            ::protobuf::__internal::runtime::build_mini_table("$/P)P1X");
        ::protobuf::__internal::runtime::link_mini_table(
            super::aegis__types__HealthCheckResponse_msg_init.0, &[], &[]);
        ::protobuf::__internal::runtime::MiniTableInitPtr(super::aegis__types__HealthCheckResponse_msg_init.0)
      }).0
    }
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetArena for HealthCheckResponse {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for HealthCheckResponse {
  type Msg = HealthCheckResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<HealthCheckResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for HealthCheckResponse {
  type Msg = HealthCheckResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<HealthCheckResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtrMut for HealthCheckResponseMut<'_> {
  type Msg = HealthCheckResponse;
  fn get_ptr_mut(&mut self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<HealthCheckResponse> {
    self.inner.ptr_mut()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for HealthCheckResponseMut<'_> {
  type Msg = HealthCheckResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<HealthCheckResponse> {
    self.inner.ptr()
  }
}
unsafe impl ::protobuf::__internal::runtime::UpbGetMessagePtr for HealthCheckResponseView<'_> {
  type Msg = HealthCheckResponse;
  fn get_ptr(&self, _private: ::protobuf::__internal::Private) -> ::protobuf::__internal::runtime::MessagePtr<HealthCheckResponse> {
    self.inner.ptr()
  }
}

unsafe impl ::protobuf::__internal::runtime::UpbGetArena for HealthCheckResponseMut<'_> {
  fn get_arena(&mut self, _private: ::protobuf::__internal::Private) -> &::protobuf::__internal::runtime::Arena {
    self.inner.arena()
  }
}



#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Transport(i32);

#[allow(non_upper_case_globals)]
impl Transport {
  pub const Unspecified: Transport = Transport(0);
  pub const Tcp: Transport = Transport(1);
  pub const Udp: Transport = Transport(2);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "Tcp",
      2 => "Udp",
      _ => return None
    })
  }
}

impl ::std::convert::From<Transport> for i32 {
  fn from(val: Transport) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for Transport {
  fn from(val: i32) -> Transport {
    Self(val)
  }
}

impl ::std::default::Default for Transport {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for Transport {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "Transport::{}", constant_name)
    } else {
      write!(f, "Transport::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for Transport {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for Transport {}

impl ::protobuf::Proxied for Transport {
  type View<'a> = Transport;
}

impl ::protobuf::AsView for Transport {
  type Proxied = Transport;

  fn as_view(&self) -> Transport {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for Transport {
  fn into_view<'shorter>(self) -> Transport where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for Transport {
  const NAME: &'static str = "Transport";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2)
  }
}

impl ::protobuf::__internal::EntityType for Transport {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TargetSource(i32);

#[allow(non_upper_case_globals)]
impl TargetSource {
  pub const Unspecified: TargetSource = TargetSource(0);
  pub const GpsCoOccurrence: TargetSource = TargetSource(1);
  pub const GpsFirstService: TargetSource = TargetSource(2);
  pub const _6HitRl: TargetSource = TargetSource(3);
  pub const PassiveCt: TargetSource = TargetSource(4);
  pub const PassiveDns: TargetSource = TargetSource(5);
  pub const BgpSeed: TargetSource = TargetSource(6);
  pub const Random: TargetSource = TargetSource(7);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "GpsCoOccurrence",
      2 => "GpsFirstService",
      3 => "_6HitRl",
      4 => "PassiveCt",
      5 => "PassiveDns",
      6 => "BgpSeed",
      7 => "Random",
      _ => return None
    })
  }
}

impl ::std::convert::From<TargetSource> for i32 {
  fn from(val: TargetSource) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for TargetSource {
  fn from(val: i32) -> TargetSource {
    Self(val)
  }
}

impl ::std::default::Default for TargetSource {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for TargetSource {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "TargetSource::{}", constant_name)
    } else {
      write!(f, "TargetSource::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for TargetSource {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for TargetSource {}

impl ::protobuf::Proxied for TargetSource {
  type View<'a> = TargetSource;
}

impl ::protobuf::AsView for TargetSource {
  type Proxied = TargetSource;

  fn as_view(&self) -> TargetSource {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for TargetSource {
  fn into_view<'shorter>(self) -> TargetSource where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for TargetSource {
  const NAME: &'static str = "TargetSource";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2|3|4|5|6|7)
  }
}

impl ::protobuf::__internal::EntityType for TargetSource {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DetectionMethod(i32);

#[allow(non_upper_case_globals)]
impl DetectionMethod {
  pub const Unspecified: DetectionMethod = DetectionMethod(0);
  pub const PortNumber: DetectionMethod = DetectionMethod(1);
  pub const LzrHeuristic: DetectionMethod = DetectionMethod(2);
  pub const ActiveProbe: DetectionMethod = DetectionMethod(3);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "PortNumber",
      2 => "LzrHeuristic",
      3 => "ActiveProbe",
      _ => return None
    })
  }
}

impl ::std::convert::From<DetectionMethod> for i32 {
  fn from(val: DetectionMethod) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for DetectionMethod {
  fn from(val: i32) -> DetectionMethod {
    Self(val)
  }
}

impl ::std::default::Default for DetectionMethod {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for DetectionMethod {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "DetectionMethod::{}", constant_name)
    } else {
      write!(f, "DetectionMethod::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for DetectionMethod {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for DetectionMethod {}

impl ::protobuf::Proxied for DetectionMethod {
  type View<'a> = DetectionMethod;
}

impl ::protobuf::AsView for DetectionMethod {
  type Proxied = DetectionMethod;

  fn as_view(&self) -> DetectionMethod {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for DetectionMethod {
  fn into_view<'shorter>(self) -> DetectionMethod where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for DetectionMethod {
  const NAME: &'static str = "DetectionMethod";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2|3)
  }
}

impl ::protobuf::__internal::EntityType for DetectionMethod {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HostStatus(i32);

#[allow(non_upper_case_globals)]
impl HostStatus {
  pub const Unspecified: HostStatus = HostStatus(0);
  pub const Active: HostStatus = HostStatus(1);
  pub const PendingEviction: HostStatus = HostStatus(2);
  pub const Evicted: HostStatus = HostStatus(3);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "Active",
      2 => "PendingEviction",
      3 => "Evicted",
      _ => return None
    })
  }
}

impl ::std::convert::From<HostStatus> for i32 {
  fn from(val: HostStatus) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for HostStatus {
  fn from(val: i32) -> HostStatus {
    Self(val)
  }
}

impl ::std::default::Default for HostStatus {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for HostStatus {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "HostStatus::{}", constant_name)
    } else {
      write!(f, "HostStatus::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for HostStatus {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for HostStatus {}

impl ::protobuf::Proxied for HostStatus {
  type View<'a> = HostStatus;
}

impl ::protobuf::AsView for HostStatus {
  type Proxied = HostStatus;

  fn as_view(&self) -> HostStatus {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for HostStatus {
  fn into_view<'shorter>(self) -> HostStatus where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for HostStatus {
  const NAME: &'static str = "HostStatus";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2|3)
  }
}

impl ::protobuf::__internal::EntityType for HostStatus {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GeoTier(i32);

#[allow(non_upper_case_globals)]
impl GeoTier {
  pub const Unspecified: GeoTier = GeoTier(0);
  pub const Cryptographic: GeoTier = GeoTier(1);
  pub const Rtt: GeoTier = GeoTier(2);
  pub const Passive: GeoTier = GeoTier(3);

  fn constant_name(&self) -> ::std::option::Option<&'static str> {
    #[allow(unreachable_patterns)] // In the case of aliases, just emit them all and let the first one match.
    Some(match self.0 {
      0 => "Unspecified",
      1 => "Cryptographic",
      2 => "Rtt",
      3 => "Passive",
      _ => return None
    })
  }
}

impl ::std::convert::From<GeoTier> for i32 {
  fn from(val: GeoTier) -> i32 {
    val.0
  }
}

impl ::std::convert::From<i32> for GeoTier {
  fn from(val: i32) -> GeoTier {
    Self(val)
  }
}

impl ::std::default::Default for GeoTier {
  fn default() -> Self {
    Self(0)
  }
}

impl ::std::fmt::Debug for GeoTier {
  fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
    if let Some(constant_name) = self.constant_name() {
      write!(f, "GeoTier::{}", constant_name)
    } else {
      write!(f, "GeoTier::from({})", self.0)
    }
  }
}

impl ::protobuf::IntoProxied<i32> for GeoTier {
  fn into_proxied(self, _: ::protobuf::__internal::Private) -> i32 {
    self.0
  }
}

impl ::protobuf::__internal::SealedInternal for GeoTier {}

impl ::protobuf::Proxied for GeoTier {
  type View<'a> = GeoTier;
}

impl ::protobuf::AsView for GeoTier {
  type Proxied = GeoTier;

  fn as_view(&self) -> GeoTier {
    *self
  }
}

impl<'msg> ::protobuf::IntoView<'msg> for GeoTier {
  fn into_view<'shorter>(self) -> GeoTier where 'msg: 'shorter {
    self
  }
}

// SAFETY: this is an enum type
unsafe impl ::protobuf::__internal::Enum for GeoTier {
  const NAME: &'static str = "GeoTier";

  fn is_known(value: i32) -> bool {
    matches!(value, 0|1|2|3)
  }
}

impl ::protobuf::__internal::EntityType for GeoTier {
    type Tag = ::protobuf::__internal::entity_tag::EnumTag;
}


