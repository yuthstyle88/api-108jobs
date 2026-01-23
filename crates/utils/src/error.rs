use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::{Display, EnumIter};

#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumIter, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "error", content = "message", rename_all = "camelCase")]
#[non_exhaustive]
pub enum FastJobErrorType {
  BlockKeywordTooShort,
  BlockKeywordTooLong,
  CouldntUpdateKeywords,
  ReportReasonRequired,
  ReportTooLong,
  NotAModerator,
  EmailNotFound,
  NotAnAdmin,
  CantBlockYourself,
  CantNoteYourself,
  CantBlockAdmin,
  CouldntUpdateUser,
  PasswordsDoNotMatch,
  EmailNotVerified,
  EmailRequired,
  CouldntUpdateComment,
  CannotLeaveAdmin,
  PictrsResponseError(String),
  PictrsPurgeResponseError(String),
  ImageUrlMissingPathSegments,
  ImageUrlMissingLastPathSegment,
  PictrsApiKeyNotProvided,
  NoContentTypeHeader,
  NotAnImageType,
  InvalidImageUpload,
  ImageUploadDisabled,
  NotAModOrAdmin,
  NotTopMod,
  NotLoggedIn,
  NotHigherMod,
  NotHigherAdmin,
  SiteBan,
  Deleted,
  PersonIsBlocked,
  CategoryIsBlocked,
  InstanceIsBlocked,
  InstanceIsPrivate,
  /// Password must be between 10 and 60 characters
  InvalidPassword,
  EmptyUsername,
  EmptyPassword,
  InvalidPasswordLength,
  SiteDescriptionLengthOverflow,
  HoneypotFailed,
  RegistrationApplicationIsPending,
  Locked,
  CouldntCreateComment,
  MaxCommentDepthReached,
  NoCommentEditAllowed,
  OnlyAdminsCanCreateCommunities,
  CategoryAlreadyExists,
  LanguageNotAllowed,
  CouldntUpdateLanguages,
  CouldntUpdatePost,
  NoPostEditAllowed,
  NsfwNotAllowed,
  EditPrivateMessageNotAllowed,
  SiteAlreadyExists,
  ApplicationQuestionRequired,
  AcceptTermsRequired,
  InvalidDefaultPostListingType,
  RegistrationClosed,
  RegistrationApplicationAnswerRequired,
  RegistrationUsernameRequired,
  EmailAlreadyExists,
  RequireVerification,
  UsernameAlreadyExists,
  PersonIsBannedFromCategory,
  NoIdGiven,
  IncorrectLogin,
  NoEmailSetup,
  LocalSiteNotSetup,
  InvalidEmailAddress(String),
  InvalidEmail,
  InvalidName,
  InvalidCodeVerifier,
  InvalidDisplayName,
  InvalidMatrixId,
  InvalidPostTitle,
  InvalidBodyField,
  BioLengthOverflow,
  AltTextLengthOverflow,
  MissingTotpToken,
  MissingTotpSecret,
  IncorrectTotpToken,
  CouldntParseTotpSecret,
  CouldntGenerateTotp,
  TotpAlreadyEnabled,
  CouldntLikeComment,
  CouldntSaveComment,
  CouldntCreateReport,
  CouldntResolveReport,
  CategoryModeratorAlreadyExists,
  CategoryUserAlreadyBanned,
  CategoryBlockAlreadyExists,
  CategoryFollowerAlreadyExists,
  PersonBlockAlreadyExists,
  CouldntLikePost,
  CouldntSavePost,
  CouldntMarkPostAsRead,
  CouldntUpdateReadComments,
  CouldntHidePost,
  CouldntUpdateCategory,
  CouldntCreatePersonCommentMention,
  CouldntUpdatePersonCommentMention,
  CouldntCreatePersonPostMention,
  CouldntUpdatePersonPostMention,
  CouldntCreatePost,
  CouldntCreatePrivateMessage,
  CouldntUpdatePrivateMessage,
  BlockedUrl,
  InvalidUrl,
  EmailSendFailed,
  Slurs,
  RegistrationDenied {
    #[cfg_attr(feature = "ts-rs", ts(optional))]
    reason: Option<String>,
  },
  SiteNameRequired,
  SiteNameLengthOverflow,
  PermissiveRegex,
  InvalidRegex,
  CaptchaIncorrect,
  CouldntCreateAudioCaptcha,
  CouldntCreateImageCaptcha,
  InvalidUrlScheme,
  CouldntSendWebmention,
  ContradictingFilters,
  InstanceBlockAlreadyExists,
  /// Thrown when an API call is submitted with more than 1000 array elements, see
  /// [[MAX_API_PARAM_ELEMENTS]]
  TooManyItems,
  BanExpirationInPast,
  InvalidUnixTime,
  InvalidDateFormat,
  InvalidBotAction,
  InvalidTagName,
  TagNotIncategory,
  CantBlockLocalInstance,
  Unknown(String),
  UrlLengthOverflow,
  OauthAuthorizationInvalid,
  OauthLoginFailed,
  OauthLoginNotfound,
  OauthRegistrationClosed,
  OauthRegistrationError,
  CouldntCreateOauthProvider,
  CouldntUpdateOauthProvider,
  NotFound,
  CategoryHasNoFollowers,
  PostScheduleTimeMustBeInFuture,
  TooManyScheduledPosts,
  CannotCombineFederationBlocklistAndAllowlist,
  CouldntParsePaginationToken,
  PluginError(String),
  InvalidFetchLimit,
  CouldntCreateCommentReply,
  CouldntUpdateCommentReply,
  CouldntMarkCommentReplyAsRead,
  CouldntCreateEmoji,
  CouldntUpdateEmoji,
  CouldntCreatePerson,
  CouldntUpdatePerson,
  CouldntCreateModlog,
  CouldntUpdateModlog,
  CouldntCreateSite,
  CouldntUpdateSite,
  CouldntCreateRegistrationApplication,
  CouldntUpdateRegistrationApplication,
  CouldntCreateTag,
  CouldntUpdateTag,
  CouldntCreatePostTag,
  CouldntUpdatePostTag,
  CouldntCreateTagline,
  CouldntUpdateTagline,
  CouldntCreateImage,
  CouldntAllowInstance,
  CouldntBlockInstance,
  CouldntInsertActivity,
  CouldntCreateRateLimit,
  CouldntCreateCaptchaAnswer,
  CouldntCreateOauthAccount,
  CouldntCreatePasswordResetRequest,
  CouldntCreateLoginToken,
  CouldntUpdateLocalSiteUrlBlocklist,
  CouldntCreateEmailVerification,
  CouldntCreateOTPVerification,
  EmailNotificationsDisabled,
  MulticategoryUpdateWrongUser,
  CannotCombineCategoryIdAndMulticategoryId,
  MulticategoryEntryLimitReached,
  CouldntConnectDatabase,
  CouldntStartWebSocket,
  CouldntCreateChatMessage,
  CouldntUpdateChatMessage,
  CouldntCreateChatRoom,
  CouldntUpdateChatRoom,
  DatabaseError,
  CouldntCreateChatRoomMember,
  CouldntUpdateChatRoomMember,
  InvalidRoomId,
  EncryptingError,
  DecryptingError,
  DecodeError,
  InvalidKeySize,
  GenerateKeyError,
  ValidationError(String),
  FileNotFound,
  EmailAlreadyVerified,
  OauthProviderDisabled,
  CouldntCreateCategoryGroup,
  CouldntUpdateCategoryGroup,
  EmptyEmail,
  MissingCaptchaUuid,
  MissingCaptchaAnswer,
  RoleNotFound,
  EmptyUsernameOrEmail,
  UserNotFound,
  EmptyTitle,
  SlugAlreadyExists,
  MaxCategoryDepthReached,
  CouldntCreateCategory,
  AlreadyDeleted,
  UrlWithoutDomain,
  FederationDisabledByStrictAllowList,
  RedisConnectionFailed,
  SerializationFailed,
  RedisSetFailed,
  RedisDeleteFailed,
  RedisKeyNotFound,
  RedisGetFailed,
  DeserializationFailed,
  PostIsLocked,
  CantDeleteSite,
  PageDoesNotSpecifyCreator,
  ObjectIsNotPublic,
  FederationDisabled,
  InvalidField(String),
  // Wallet related errors
  WalletAlreadyExists,
  WalletNotFound,
  InsufficientBalance,
  PostNameAlreadyExists,
  // Contact related errors
  CouldntCreateContact,
  CouldntCreateAddress,
  CouldntUpdateContact,
  CouldntFindContact,
  CouldntDeleteContact,
  CouldntDeleteAddress,
  CouldntFindIdentityCard,
  CouldntFindAddress,
  CouldntCreateIdentityCard,
  CouldntUpdateIdentityCard,
  CouldntDeleteIdentityCard,
  InvalidIssueAndExpire,
  CouldntFindWalletByUser,
  InsufficientEscrowBalance,
  CouldntCreateWallet,
  CouldntUpdateWallet,
  EmptyIDNumber,
  EmptyNationality,
  InvalidIDNumber,
  EmpltyFullName,
  MissingLocalUserId,
  IDNumberAlreadyExist,
  EmptyAddressLine1,
  EmptySubdistrict,
  EmptyDistrict,
  EmptyProvince,
  EmptyPostalCode,
  EmptyCountryID,
  InvalidCountryID,
  CouldntCreateSkill,
  CouldntUpdateSkill,
  CouldntCreateWorkExperience,
  CouldntUpdateWorkExperience,
  CouldntCreateEducation,
  CouldntUpdateEducation,
  SkillCouldntEmpty,
  CouldntDeleteBankAccount,
  CouldntUpdateBankAccount,
  CouldntUpdateBilling,
  CouldntUpdateWalletTranSaction,
  CouldntCreateWalletTranSaction,
  NegativeAmount,
  CouldntDeleteEducation,
  CouldntDeleteWorkExperience,
  CouldntDeleteCertificate,
  CouldntDeleteLanguageProfile,
  NotAllowed,
  NoSiteConfig,
  NoAdmin,
  DuplicateIDxInInstallments,
  StatusPaidOrUnpaid,
  NegativeIDx,
  CannotCommentOnOwnPost,
  AlreadyCommented,
  CouldntCreateChatParticipant,
  CouldntUpdateChatParticipant,
  CouldntListRoomForUser,
  CouldntEnsureParticipant,
  CouldntDeleteFile,
  InsufficientBalanceForTransfer,
  InsufficientBalanceForWithdraw,
  WalletInvariantViolated,
  StillDoNotPayYet,
  ReturnedNonJSONResponse,
  CouldntSaveLastRead,
  InvalidInput(String),
  CiphertextTooShort,
  InvalidLength,
  InvalidArgument,
  EncodeError,
  InvalidAlgorithm,
  InvalidKeyLength,
  InvalidIVLength,
  LastReadNotFound,
  WorkflowDoesNotExist,
    CouldntCreatePendingSenderAck,
  CouldntUpdatePendingSenderAck,
    BankAccountAlreadyExistsForThisBank,
    ExternalApiError,
    UnauthorizedAccess,
  CannotDeleteDefaultBankAccount,
    ReachedMax3BankAccounts,
    CouldntUpdateChatUnread,
    RedisPipelineFailed,
    CouldntCreateRider,
  CouldntUpdateRider,
  RiderAlreadyExists,
    InvalidData,
  CouldntCreateDeliveryLocation,
  CouldntUpdateDeliveryLocation,
  CouldntCreateDeliveryLocationHistory,
  InvalidLatitudeOrLongitude,
  CouldntCreateDeliveryDetails,
  CouldntUpdateDeliveryDetails,
}

cfg_if! {
  if #[cfg(feature = "full")] {

    use std::{fmt, backtrace::Backtrace};
    pub type FastJobResult<T> = Result<T, FastJobError>;

    pub struct FastJobError {
      pub error_type: FastJobErrorType,
      pub inner: anyhow::Error,
      pub context: Backtrace,
    }

    /// Maximum number of items in an array passed as API parameter. See [[FastJobErrorType::TooManyItems]]
    pub(crate) const MAX_API_PARAM_ELEMENTS: usize = 10_000;

    impl<T> From<T> for FastJobError
    where
      T: Into<anyhow::Error>,
    {
      fn from(t: T) -> Self {
        let cause = t.into();
        let error_type = match cause.downcast_ref::<diesel::result::Error>() {
          Some(&diesel::NotFound) => FastJobErrorType::NotFound,
          _ => FastJobErrorType::Unknown(format!("{}", &cause))
      };
        FastJobError {
          error_type,
          inner: cause,
          context: Backtrace::capture(),
        }
      }
    }

    impl Debug for FastJobError {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FastJobError")
         .field("message", &self.error_type)
         .field("inner", &self.inner)
         .field("context", &self.context)
         .finish()
      }
    }

    impl fmt::Display for FastJobError {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.error_type)?;
        writeln!(f, "{}", self.inner)?;
        fmt::Display::fmt(&self.context, f)
      }
    }

    impl actix_web::error::ResponseError for FastJobError {
      fn status_code(&self) -> actix_web::http::StatusCode {
        match self.error_type {
          FastJobErrorType::IncorrectLogin => actix_web::http::StatusCode::UNAUTHORIZED,
          FastJobErrorType::NotFound => actix_web::http::StatusCode::NOT_FOUND,
          _ => actix_web::http::StatusCode::BAD_REQUEST,
        }
      }

      fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(&self.error_type)
      }
    }

    impl From<FastJobErrorType> for FastJobError {
      fn from(error_type: FastJobErrorType) -> Self {
        let inner = anyhow::anyhow!("{}", error_type);
        FastJobError {
          error_type,
          inner,
          context: Backtrace::capture(),
        }
      }
    }


    pub trait FastJobErrorExt<T, E: Into<anyhow::Error>> {
      fn with_fastjob_type(self, error_type: FastJobErrorType) -> FastJobResult<T>;
    }

    impl<T, E: Into<anyhow::Error>> FastJobErrorExt<T, E> for Result<T, E> {
      fn with_fastjob_type(self, error_type: FastJobErrorType) -> FastJobResult<T> {
        self.map_err(|error| FastJobError {
          error_type,
          inner: error.into(),
          context: Backtrace::capture(),
        })
      }
    }
    pub trait FastJobErrorExt2<T> {
      fn with_fastjob_type(self, error_type: FastJobErrorType) -> FastJobResult<T>;
      fn into_anyhow(self) -> Result<T, anyhow::Error>;
    }

    impl<T> FastJobErrorExt2<T> for FastJobResult<T> {
      fn with_fastjob_type(self, error_type: FastJobErrorType) -> FastJobResult<T> {
        self.map_err(|mut e| {
          e.error_type = error_type;
          e
        })
      }
      // this function can't be an impl From or similar because it would conflict with one of the other broad Into<> implementations
      fn into_anyhow(self) -> Result<T, anyhow::Error> {
        self.map_err(|e| e.inner)
      }
    }

    #[cfg(test)]
    mod tests {
      #![allow(clippy::indexing_slicing)]
      use super::*;
      use actix_web::{body::MessageBody, ResponseError};
      use pretty_assertions::assert_eq;
      use std::fs::read_to_string;
      use strum::IntoEnumIterator;

      #[test]
      fn deserializes_no_message() -> FastJobResult<()> {
        let err = FastJobError::from(FastJobErrorType::BlockedUrl).error_response();
        let json = String::from_utf8(err.into_body().try_into_bytes().unwrap_or_default().to_vec())?;
        assert_eq!(&json, "{\"error\":\"blocked_url\"}");

        Ok(())
      }

      #[test]
      fn deserializes_with_message() -> FastJobResult<()> {
        let reg_banned = FastJobErrorType::PictrsResponseError(String::from("reason"));
        let err = FastJobError::from(reg_banned).error_response();
        let json = String::from_utf8(err.into_body().try_into_bytes().unwrap_or_default().to_vec())?;
        assert_eq!(
          &json,
          "{\"error\":\"pictrs_response_error\",\"message\":\"reason\"}"
        );

        Ok(())
      }

      #[test]
      fn test_convert_diesel_errors() {
        let not_found_error = FastJobError::from(diesel::NotFound);
        assert_eq!(FastJobErrorType::NotFound, not_found_error.error_type);
        assert_eq!(404, not_found_error.status_code());

        let other_error = FastJobError::from(diesel::result::Error::NotInTransaction);
        assert!(matches!(other_error.error_type, FastJobErrorType::Unknown{..}));
        assert_eq!(400, other_error.status_code());
      }

      /// Check if errors match translations. Disabled because many are not translated at all.
      #[test]
      #[ignore]
      fn test_translations_match() -> FastJobResult<()> {
        #[derive(Deserialize)]
        struct Err {
          error: String,
        }

        let translations = read_to_string("translations/translations/en.json")?;

        for e in FastJobErrorType::iter() {
          let msg = serde_json::to_string(&e)?;
          let msg: Err = serde_json::from_str(&msg)?;
          let msg = msg.error;
          assert!(translations.contains(&format!("\"{msg}\"")), "{msg}");
        }

        Ok(())
      }
    }
  }
}
