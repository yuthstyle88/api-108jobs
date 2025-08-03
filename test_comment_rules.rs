// Simple test to verify comment business rules
// This is just for documentation - the actual validation happens in create.rs

/*
New Comment Business Rules:

1. Users cannot comment on their own posts
   - Check: post.creator_id == local_user_view.person.id
   - Error: "cannot comment on your own post"

2. Users can only comment once per post
   - Check: Query existing comments by user on this post
   - Filter: deleted=false, removed=false
   - Error: "You have already commented on this post"

3. Comments are flat (no replies)
   - All comments are top-level
   - No parent_id or tree structure

Implementation in create.rs:
- Added validation before comment creation
- Database query to check existing comments
- Early return with descriptive error messages
*/

fn main() {
    println!("Comment validation rules implemented in crates/api/api_crud/src/comment/create.rs");
}