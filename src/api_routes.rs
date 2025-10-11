use actix_web::{guard, web::*};
use lemmy_api::admin::bank_account::list_bank_accounts;
use lemmy_api::admin::wallet::{admin_top_up_wallet, admin_withdraw_wallet};
use lemmy_api::chat::list::list_chat_rooms;
use lemmy_api::local_user::bank_account::{
  create_bank_account, delete_bank_account, list_banks, list_user_bank_accounts,
  set_default_bank_account,
};
use lemmy_api::local_user::exchange::{exchange_key, get_user_keys};
use lemmy_api::local_user::profile::visit_profile;
use lemmy_api::local_user::review::{list_user_reviews, submit_user_review};
use lemmy_api::local_user::update_term::update_term;
use lemmy_api::local_user::wallet::get_wallet;
use lemmy_api::local_user::workflow::{
  approve_quotation, approve_work, cancel_job, create_quotation, get_billing_by_room,
  request_revision, start_workflow, submit_start_work, submit_work, update_budget_plan_status,
};
use lemmy_api::{
  comment::{
    distinguish::distinguish_comment, like::like_comment, list_comment_likes::list_comment_likes,
    save::save_comment,
  },
  community::{
    random::get_random_community,
    tag::{create_community_tag, delete_community_tag, update_community_tag},
  },
  local_user::{
    add_admin::add_admin,
    ban_person::ban_from_site,
    change_password::change_password,
    change_password_after_reset::change_password_after_reset,
    donation_dialog_shown::donation_dialog_shown,
    export_data::export_data,
    generate_totp_secret::generate_totp_secret,
    get_captcha::get_captcha,
    list_created::list_person_created,
    list_hidden::list_person_hidden,
    list_liked::list_person_liked,
    list_logins::list_logins,
    list_media::list_media,
    list_read::list_person_read,
    list_saved::list_person_saved,
    login::login,
    logout::logout,
    note_person::user_note_person,
    notifications::{
      list_inbox::list_inbox, mark_all_read::mark_all_notifications_read,
      mark_comment_mention_read::mark_comment_mention_as_read,
      mark_post_mention_read::mark_post_mention_as_read, mark_reply_read::mark_reply_as_read,
      unread_count::unread_count,
    },
    report_count::report_count,
    resend_verification_email::resend_verification_email,
    reset_password::reset_password,
    save_settings::save_user_settings,
    update_totp::update_totp,
    validate_auth::validate_auth,
    verify_email::verify_email,
  },
  post::{
    feature::feature_post, get_link_metadata::get_link_metadata, hide::hide_post, like::like_post,
    list_post_likes::list_post_likes, lock::lock_post, mark_many_read::mark_posts_as_read,
    mark_read::mark_post_as_read, save::save_post, update_notifications::update_post_notifications,
  },
  reports::{
    comment_report::{create::create_comment_report, resolve::resolve_comment_report},
    community_report::{create::create_community_report, resolve::resolve_community_report},
    report_combined::list::list_reports,
  },
  site::{
    admin_allow_instance::admin_allow_instance,
    admin_block_instance::admin_block_instance,
    admin_list_users::admin_list_users,
    leave_admin::leave_admin,
    list_all_media::list_all_media,
    mod_log::get_mod_log,
    purge::{comment::purge_comment, person::purge_person, post::purge_post},
    registration_applications::{
      approve::approve_registration_application, get::get_registration_application,
      list::list_registration_applications,
      unread_count::get_unread_registration_application_count,
    },
  },
};
use lemmy_api_crud::chat::create::create_chat_room;
use lemmy_api_crud::chat::read::get_chat_room;
use lemmy_api_crud::community::list::list_communities;
use lemmy_api_crud::oauth_provider::create::create_oauth_provider;
use lemmy_api_crud::oauth_provider::delete::delete_oauth_provider;
use lemmy_api_crud::oauth_provider::update::update_oauth_provider;
use lemmy_api_crud::{
  comment::{
    create::create_comment, delete::delete_comment, read::get_comment, remove::remove_comment,
    update::update_comment,
  },
  community::update::update_community,
  custom_emoji::{
    create::create_custom_emoji, delete::delete_custom_emoji, list::list_custom_emojis,
    update::update_custom_emoji,
  },
  post::{
    create::create_post, delete::delete_post, read::get_post, remove::remove_post,
    update::update_post,
  },
  site::{create::create_site, read::get_site, update::update_site},
  tagline::{
    create::create_tagline, delete::delete_tagline, list::list_taglines, update::update_tagline,
  },
  user::{
    create::{authenticate_with_oauth, register},
    delete::delete_account,
    my_user::get_my_user,
  },
};
use lemmy_apub::api::list_comments::list_comments;
use lemmy_apub::api::list_posts::list_posts;
use lemmy_apub::api::search::search;
use lemmy_routes::files::delete::delete_file;
use lemmy_routes::files::download::get_file;
use lemmy_routes::files::upload::upload_file;
use lemmy_routes::images::{
  delete::{
    delete_community_banner, delete_community_icon, delete_image, delete_image_admin,
    delete_site_banner, delete_site_icon, delete_user_avatar, delete_user_banner,
  },
  download::{get_image, image_proxy},
  pictrs_health,
  upload::{
    upload_community_banner, upload_community_icon, upload_image, upload_site_banner,
    upload_site_icon, upload_user_avatar, upload_user_banner,
  },
};
use lemmy_routes::payments::create_qrcode::create_qrcode;
use lemmy_routes::payments::get_token::generate_scb_token;
use lemmy_routes::payments::inquire::inquire_qrcode;
use lemmy_utils::rate_limit::RateLimit;
use lemmy_ws::handler::{get_history, get_last_read, phoenix_ws};

pub fn config(cfg: &mut ServiceConfig, rate_limit: &RateLimit) {
  cfg
    .service(resource("/socket/websocket").route(get().to(phoenix_ws)))
    .service(
      scope("/api/v4")
        // .wrap(rate_limit.message())
        // Site
        .service(
          scope("/site")
            .route("", get().to(get_site))
            .route("", post().to(create_site))
            .route("", put().to(update_site))
            .route("/icon", post().to(upload_site_icon))
            .route("/icon", delete().to(delete_site_icon))
            .route("/banner", post().to(upload_site_banner))
            .route("/banner", delete().to(delete_site_banner)),
        )
        .route("/modlog", get().to(get_mod_log))
        .service(
          resource("/search")
            // .wrap(rate_limit.search())
            .route(get().to(search)),
        )
        // Community
        .service(
          resource("/community")
            .guard(guard::Post())
            .wrap(rate_limit.register()),
        )
        .service(
          scope("/community")
            .route("", put().to(update_community))
            .route("/random", get().to(get_random_community))
            .route("/list", get().to(list_communities))
            .route("/report", post().to(create_community_report))
            .route("/report/resolve", put().to(resolve_community_report))
            // Mod Actions
            .route("/icon", post().to(upload_community_icon))
            .route("/icon", delete().to(delete_community_icon))
            .route("/banner", post().to(upload_community_banner))
            .route("/banner", delete().to(delete_community_banner))
            .route("/tag", post().to(create_community_tag))
            .route("/tag", put().to(update_community_tag))
            .route("/tag", delete().to(delete_community_tag)),
        )
        // Post
        .service(
          resource("/post")
            // Handle POST to /post separately to add the post() rate limitter
            .guard(guard::Post())
            // .wrap(rate_limit.post())
            .route(post().to(create_post)),
        )
        .service(
          resource("/post/site-metadata")
            .wrap(rate_limit.search())
            .route(get().to(get_link_metadata)),
        )
        .service(
          scope("/post")
            .route("", get().to(get_post))
            .route("", put().to(update_post))
            .route("/delete", post().to(delete_post))
            .route("/remove", post().to(remove_post))
            .route("/mark-as-read", post().to(mark_post_as_read))
            .route("/mark-as-read/many", post().to(mark_posts_as_read))
            .route("/hide", post().to(hide_post))
            .route("/lock", post().to(lock_post))
            .route("/feature", post().to(feature_post))
            .route("/list", get().to(list_posts))
            .route("/like", post().to(like_post))
            .route("/like/list", get().to(list_post_likes))
            .route("/save", put().to(save_post))
            .route(
              "/disable-notifications",
              post().to(update_post_notifications),
            ),
        )
        // Comment
        .service(
          // Handle POST to /comment separately to add the comment() rate limitter
          resource("/comment")
            .guard(guard::Post())
            // .wrap(rate_limit.comment())
            .route(post().to(create_comment)),
        )
        .service(
          scope("/comment")
            .route("", get().to(get_comment))
            .route("", put().to(update_comment))
            .route("/delete", post().to(delete_comment))
            .route("/remove", post().to(remove_comment))
            .route("/mark-as-read", post().to(mark_reply_as_read))
            .route("/distinguish", post().to(distinguish_comment))
            .route("/like", post().to(like_comment))
            .route("/like/list", get().to(list_comment_likes))
            .route("/list", get().to(list_comments))
            .route("/save", put().to(save_comment))
            .route("/report", post().to(create_comment_report))
            .route("/report/resolve", put().to(resolve_comment_report)),
        )
        // Reports
        .service(
          scope("/report")
            .wrap(rate_limit.message())
            .route("/list", get().to(list_reports)),
        )
        // User
        .service(
          scope("/account/auth")
            // .wrap(rate_limit.register())
            .route("/register", post().to(register))
            .route("/login", post().to(login))
            .route("/logout", post().to(logout))
            .route("/password-reset", post().to(reset_password))
            .route("/password-change", post().to(change_password_after_reset))
            .route("/change-password", put().to(change_password))
            .route("/totp/generate", post().to(generate_totp_secret))
            .route("/totp/update", post().to(update_totp))
            .route("/verify-email", post().to(verify_email))
            .route("/exchange-public-key", post().to(exchange_key))
            .route("/update-term", post().to(update_term))
            .route("/get-captcha", get().to(get_captcha))
            .route(
              "/resend-verification-email",
              post().to(resend_verification_email),
            ),
        )
        .route("/files/{user_id}/{filename}", get().to(get_file))
        .service(
          scope("/account")
            .route("", get().to(get_my_user))
            .route("/profile/{username}", get().to(visit_profile))
            .service(
              scope("/media")
                .route("", delete().to(delete_image))
                .route("/list", get().to(list_media)),
            )
            .route("/inbox", get().to(list_inbox))
            .route("/delete", post().to(delete_account))
            // upload file
            .service(
              scope("/files")
                .route("", post().to(upload_file))
                .route("/{filename}", delete().to(delete_file)),
            )
            .service(
              scope("/bank-account")
                .route("", post().to(create_bank_account))
                .route("", get().to(list_user_bank_accounts))
                .route("/default", put().to(set_default_bank_account))
                .route("/delete", post().to(delete_bank_account)),
            )
            .service(
              scope("/mention")
                .route(
                  "/comment/mark-as-read",
                  post().to(mark_comment_mention_as_read),
                )
                .route("/post/mark-as-read", post().to(mark_post_mention_as_read)),
            )
            .route("/mark-as-read/all", post().to(mark_all_notifications_read))
            .route("/report_count", get().to(report_count))
            .route("/unread-count", get().to(unread_count))
            .route("/list-logins", get().to(list_logins))
            .route("/validate-auth", get().to(validate_auth))
            .route("/donation-dialog-shown", post().to(donation_dialog_shown))
            .route("/avatar", post().to(upload_user_avatar))
            .route("/avatar", delete().to(delete_user_avatar))
            .route("/banner", post().to(upload_user_banner))
            .route("/banner", delete().to(delete_user_banner))
            .route("/saved", get().to(list_person_saved))
            .route("/created", get().to(list_person_created))
            .route("/read", get().to(list_person_read))
            .route("/hidden", get().to(list_person_hidden))
            .route("/liked", get().to(list_person_liked))
            .route("/settings/save", put().to(save_user_settings))
            .route("/reviews", post().to(submit_user_review))
            // Wallet service scope
            .service(scope("/wallet").route("", get().to(get_wallet)))
            // Bank account management scope
            .service(scope("/banks").route("", get().to(list_banks)))
            // Services scope for workflow service
            .service(
              scope("/services")
                .route("/create-invoice", post().to(create_quotation))
                .route("/approve-quotation", post().to(approve_quotation))
                .route("/start-workflow", post().to(start_workflow))
                .route("/start-work", post().to(submit_start_work))
                .route("/submit-work", post().to(submit_work))
                .route("/request-revision", post().to(request_revision))
                .route("/approve-work", post().to(approve_work))
                .route("/budget-plan", put().to(update_budget_plan_status))
                .route("/billing/by-room", get().to(get_billing_by_room))
                .route("/cancel-job", post().to(cancel_job)),
            )
            // Account settings import / export have a strict rate limit
            .service(scope("/settings").wrap(rate_limit.import_user_settings()))
            .service(
              resource("/data/export")
                .wrap(rate_limit.import_user_settings())
                .route(get().to(export_data)),
            ),
        )
        // User actions
        .service(scope("/person").route("/note", post().to(user_note_person)))
        // Admin Actions
        .service(
          scope("/admin")
            .route("/add", post().to(add_admin))
            .route(
              "/registration-application/count",
              get().to(get_unread_registration_application_count),
            )
            .route(
              "/registration-application/list",
              get().to(list_registration_applications),
            )
            .route(
              "/registration-application/approve",
              put().to(approve_registration_application),
            )
            .route(
              "/registration-application",
              get().to(get_registration_application),
            )
            .service(
              scope("/purge")
                .route("/person", post().to(purge_person))
                .route("/post", post().to(purge_post))
                .route("/comment", post().to(purge_comment)),
            )
            .service(
              scope("/tagline")
                .route("", post().to(create_tagline))
                .route("", put().to(update_tagline))
                .route("/delete", post().to(delete_tagline))
                .route("/list", get().to(list_taglines)),
            )
            .route("/ban", post().to(ban_from_site))
            .route("/users", get().to(admin_list_users))
            .route("/leave", post().to(leave_admin))
            .service(
              scope("/instance")
                .route("/block", post().to(admin_block_instance))
                .route("/allow", post().to(admin_allow_instance)),
            )
            .service(
              scope("/wallet")
                .route("/top-up", post().to(admin_top_up_wallet))
                .route("/withdraw", post().to(admin_withdraw_wallet)),
            )
            .service(scope("/bank-account").route("/list", get().to(list_bank_accounts))),
        )
        .service(
          scope("/chat")
            .route("/history", get().to(get_history))
            .route("/rooms", get().to(list_chat_rooms))
            .route("/rooms", post().to(create_chat_room))
            .route("/rooms/{id}", get().to(get_chat_room))
            .route("/last-read", get().to(get_last_read)),
        )
        .service(
          scope("/users")
            .route("/{id}/keys", get().to(get_user_keys))
            .route("/{id}/reviews", get().to(list_user_reviews)),
        )
        .service(
          scope("/custom-emoji")
            .route("", post().to(create_custom_emoji))
            .route("", put().to(update_custom_emoji))
            .route("/delete", post().to(delete_custom_emoji))
            .route("/list", get().to(list_custom_emojis)),
        )
        .service(
          scope("/oauth-provider")
            .route("", post().to(create_oauth_provider))
            .route("", put().to(update_oauth_provider))
            .route("/delete", post().to(delete_oauth_provider)),
        )
        .service(
          scope("/oauth")
            // .wrap(rate_limit.register())
            .route("/authenticate", post().to(authenticate_with_oauth)),
        )
        .service(
          scope("/image")
            .service(
              resource("")
                .wrap(rate_limit.image())
                .route(post().to(upload_image))
                .route(delete().to(delete_image_admin)),
            )
            .route("/proxy", get().to(image_proxy))
            .route("/health", get().to(pictrs_health))
            .route("/list", get().to(list_all_media))
            .route("/{filename}", get().to(get_image)),
        )
        //scb payment
        .service(
          scope("/scb")
            .route("/token", post().to(generate_scb_token))
            .route("/qrcode/create", post().to(create_qrcode))
            .route("/inquire", post().to(inquire_qrcode)),
        ),
    );
}
