#![cfg(test)]

use crate::*;

use primitives::{Blake2Hasher, H256};

use srml_support::{impl_outer_origin}; // assert, assert_eq
use crate::{GenesisConfig, Module, Trait};
use runtime_primitives::{
    testing::{Digest, DigestItem, Header},
    traits::{BlakeTwo256, IdentityLookup}, //OnFinalize, OnInitialize},
    BuildStorage,
};

/// Module which has a full Substrate module for 
/// mocking behaviour of MembershipRegistry
pub mod registry {

    use srml_support::*;
    use super::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
    pub struct Member<AccountId> {
        pub id: AccountId
    }

    decl_storage! {
        trait Store for Module<T: Trait> as MockForumUserRegistry {

            pub ForumUserById get(forum_user_by_id) config(): map T::AccountId => Member<T::AccountId>;

        }

        // A weird hack required to convince decl_storage that our field above
        // does indeed use the T, otherwise would start complaining about phantom fields.
        // An alternative fix is to add this extra key when initiatilising a gensisconfig for this module
        // _genesis_phantom_data: Default::default() 
        extra_genesis_skip_phantom_data_field;
    }

    decl_module! {
        pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
    }

    impl<T: Trait> Module<T> {

        pub fn get_member(account_id: &T::AccountId) -> Option<Member<T::AccountId>> { 
            
            if <ForumUserById<T>>::exists(account_id) {
                Some(<ForumUserById<T>>::get(account_id))
            } else {
                None
            }

        }

        pub fn add_member(member: & Member<T::AccountId>) {
            <ForumUserById<T>>::insert(member.id.clone(), member.clone());
        }

    }

    impl<T: Trait> ForumUserRegistry<T::AccountId> for Module<T> {

        fn get_forum_user(id: &T::AccountId) -> Option<ForumUser<T::AccountId>> {

            if <ForumUserById<T>>::exists(id) {

                let m = <ForumUserById<T>>::get(id);

                Some(ForumUser{
                    id : m.id 
                })

            } else {
                None
            }

        }
    }

    pub type TestMembershipRegistryModule = Module<Runtime>;

}

impl_outer_origin! {
    pub enum Origin for Runtime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;

impl system::Trait for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Digest = Digest;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type Log = DigestItem;
}

impl timestamp::Trait for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
}

impl Trait for Runtime {
    type Event = ();
    type MembershipRegistry = registry::TestMembershipRegistryModule;
}

#[derive(Clone)]
pub enum OriginType {
    Signed(<Runtime as system::Trait>::AccountId),
    //Inherent, <== did not find how to make such an origin yet
    Root
}

fn mock_origin(origin: OriginType) -> mock::Origin {
    match origin {
        OriginType::Signed(account_id) => Origin::signed(account_id),
        //OriginType::Inherent => Origin::inherent,
        OriginType::Root => system::RawOrigin::Root.into() //Origin::root
    }
}

pub fn generate_text(len: usize) -> Vec<u8> {
    vec![b'x'; len]
}

pub fn good_category_title() -> Vec<u8> {
    b"Great new category".to_vec()
}

pub fn good_category_description() -> Vec<u8> {
    b"This is a great new category for the forum".to_vec()
}

pub fn good_thread_title() -> Vec<u8> {
    b"Great new thread".to_vec()
}

pub fn good_thread_text() -> Vec<u8> {
    b"The first post in this thread".to_vec()
}

/*
 * These test fixtures can be heavily refactored to avoid repotition, needs macros, and event
 * assertions are also missing.
 */ 

pub struct CreateCategoryFixture {
    pub origin: OriginType,
    pub parent: Option<CategoryId>,
    pub title: Vec<u8>,
    pub description: Vec<u8>,
    pub result: dispatch::Result
}

impl CreateCategoryFixture {

    pub fn call_and_assert(&self) {
        assert_eq!(
            TestForumModule::create_category(
                mock_origin(self.origin.clone()),
                self.parent,
                self.title.clone(),
                self.description.clone()
            ),
            self.result
        )
    }
}

pub struct UpdateCategoryFixture {
    pub origin: OriginType,
    pub category_id: CategoryId,
    pub new_archival_status: Option<bool>,
    pub new_deletion_status: Option<bool>,
    pub result: dispatch::Result
}

impl UpdateCategoryFixture {

    pub fn call_and_assert(&self) {
        assert_eq!(
            TestForumModule::update_category(
                mock_origin(self.origin.clone()),
                self.category_id,
                self.new_archival_status.clone(),
                self.new_deletion_status.clone()
            ),
            self.result
        )
    }
}

pub struct CreateThreadFixture {
    pub origin: OriginType,
    pub category_id: CategoryId,
    pub title: Vec<u8>,
    pub text: Vec<u8>,
    pub result: dispatch::Result
}

impl CreateThreadFixture {

    pub fn call_and_assert(&self) {
        assert_eq!(
            TestForumModule::create_thread(
                mock_origin(self.origin.clone()),
                self.category_id,
                self.title.clone(),
                self.text.clone()
            ),
            self.result
        )
    }
}

pub struct CreatePostFixture {
    pub origin: OriginType,
    pub thread_id: ThreadId,
    pub text: Vec<u8>,
    pub result: dispatch::Result
}

impl CreatePostFixture {

    pub fn call_and_assert(&self) {
        assert_eq!(
            TestForumModule::add_post(
                mock_origin(self.origin.clone()),
                self.thread_id,
                self.text.clone()
            ),
            self.result
        )
    }
}

pub fn create_forum_member() -> OriginType {
    let member_id = 123;
    let new_member = registry::Member { id: member_id };
    registry::TestMembershipRegistryModule::add_member(&new_member);
    OriginType::Signed(member_id)
}

pub fn create_category(forum_sudo: OriginType, parent_category_id: Option<CategoryId>) -> CategoryId {
    let category_id = TestForumModule::next_category_id();
    CreateCategoryFixture {
        origin: forum_sudo,
        parent: parent_category_id,
        title: good_category_title(),
        description: good_category_description(),
        result: Ok(())
    }
    .call_and_assert();
    category_id
}

pub fn create_root_category(forum_sudo: OriginType) -> CategoryId {
    create_category(forum_sudo, None)
}

pub fn create_root_category_and_thread(forum_sudo: OriginType) -> (OriginType, CategoryId, ThreadId) {
    let member_origin = create_forum_member();
    let category_id = create_root_category(forum_sudo);
    let thread_id = TestForumModule::next_thread_id();

    CreateThreadFixture {
        origin: member_origin.clone(),
        category_id,
        title: good_thread_title(),
        text: good_thread_text(),
        result: Ok(())
    }
    .call_and_assert();

    (member_origin, category_id, thread_id)
}

pub fn moderate_thread(forum_sudo: OriginType, thread_id: ThreadId, rationale: Vec<u8>) -> dispatch::Result {
    TestForumModule::moderate_thread(
        mock_origin(forum_sudo), thread_id, rationale
    )
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.

// refactor
/// - add each config as parameter, then 
/// 

pub fn default_genesis_config() -> GenesisConfig<Runtime> {

    GenesisConfig::<Runtime> {
        category_by_id: vec![], // endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
        next_category_id: 1,
        thread_by_id: vec![],
        next_thread_id: 1,
        post_by_id: vec![],
        next_post_id: 1,

        forum_sudo: 33,

        category_title_constraint: InputValidationLengthConstraint{
            min: 10,
            max_min_diff: 140
        },

        category_description_constraint: InputValidationLengthConstraint{
            min: 10,
            max_min_diff: 140
        },

        thread_title_constraint: InputValidationLengthConstraint{
            min: 3,
            max_min_diff: 43
        },

        post_text_constraint: InputValidationLengthConstraint{
            min: 1,
            max_min_diff: 1001
        },

        thread_moderation_rationale_constraint: InputValidationLengthConstraint{
            min: 100,
            max_min_diff: 2000
        },

        post_moderation_rationale_constraint: InputValidationLengthConstraint{
            min: 100,
            max_min_diff: 2000
        }


        // JUST GIVING UP ON ALL THIS FOR NOW BECAUSE ITS TAKING TOO LONG

        // Extra genesis fields
        //initial_forum_sudo: Some(143)
    }
}

pub type RuntimeMap<K,V> = std::vec::Vec<(K, V)>;
pub type RuntimeCategory = Category< <Runtime as system::Trait>::BlockNumber, <Runtime as timestamp::Trait>::Moment, <Runtime as system::Trait>::AccountId>;
pub type RuntimeThread = Thread< <Runtime as system::Trait>::BlockNumber, <Runtime as timestamp::Trait>::Moment, <Runtime as system::Trait>::AccountId>;
pub type RuntimePost = Post< <Runtime as system::Trait>::BlockNumber, <Runtime as timestamp::Trait>::Moment, <Runtime as system::Trait>::AccountId>;
pub type RuntimeBlockchainTimestamp = BlockchainTimestamp<<Runtime as system::Trait>::BlockNumber, <Runtime as timestamp::Trait>::Moment>;

pub fn genesis_config(
    category_by_id: &RuntimeMap<CategoryId, RuntimeCategory>,
    next_category_id: u64,
    thread_by_id: &RuntimeMap<ThreadId, RuntimeThread>,
    next_thread_id: u64,
    post_by_id: &RuntimeMap<PostId, RuntimePost>,
    next_post_id: u64,
    forum_sudo: <Runtime as system::Trait>::AccountId,
    category_title_constraint: &InputValidationLengthConstraint,
    category_description_constraint: &InputValidationLengthConstraint,
    thread_title_constraint: &InputValidationLengthConstraint,
    post_text_constraint: &InputValidationLengthConstraint,
    thread_moderation_rationale_constraint: &InputValidationLengthConstraint,
    post_moderation_rationale_constraint: &InputValidationLengthConstraint
    ) 
    -> GenesisConfig<Runtime> {

    GenesisConfig::<Runtime> {
        category_by_id: category_by_id.clone(),
        next_category_id: next_category_id,
        thread_by_id: thread_by_id.clone(),
        next_thread_id: next_thread_id,
        post_by_id: post_by_id.clone(),
        next_post_id: next_post_id,
        forum_sudo: forum_sudo,
        category_title_constraint: category_title_constraint.clone(),
        category_description_constraint: category_description_constraint.clone(),
        thread_title_constraint: thread_title_constraint.clone(),
        post_text_constraint: post_text_constraint.clone(),
        thread_moderation_rationale_constraint: thread_moderation_rationale_constraint.clone(),
        post_moderation_rationale_constraint: post_moderation_rationale_constraint.clone()
    }
}

// MockForumUserRegistry
pub fn default_mock_forum_user_registry_genesis_config() -> registry::GenesisConfig<Runtime> {

    registry::GenesisConfig::<Runtime> {
        forum_user_by_id : vec![],
    }
}

// NB!:
// Wanted to have payload: a: &GenesisConfig<Test>
// but borrow checker made my life miserabl, so giving up for now.
pub fn build_test_externalities(config: GenesisConfig<Runtime>) -> runtime_io::TestExternalities<Blake2Hasher> {

    let mut t = config
        .build_storage()
        .unwrap()
        .0;

    // Add mock registry configuration
    t.extend(
        default_mock_forum_user_registry_genesis_config()
        .build_storage()
        .unwrap()
        .0
    );

    t.into()
}

// pub type System = system::Module<Runtime>;

/// Export forum module on a test runtime
pub type TestForumModule = Module<Runtime>;
