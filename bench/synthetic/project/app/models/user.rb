class User < ActiveRecord::Base
  # EnumSyntax: old-style enum syntax
  enum status: { active: 0, archived: 1 }
  enum role: [:admin, :user]

  # EnumUniqueness: duplicate numeric values
  enum priority: { low: 0, medium: 1, high: 0 }

  # UniqueValidationWithoutIndex
  validates :email, uniqueness: true

  # RedundantPresenceValidationOnBelongsTo
  belongs_to :organization
  validates :organization, presence: true

  # BelongsTo: required: false (should be optional: true)
  belongs_to :team, required: false

  # DelegateAllowBlank
  delegate :name, to: :organization, allow_blank: true

  # UnusedIgnoredColumns
  self.ignored_columns = [:legacy_field]

  # AfterCommitOverride: duplicate callback method
  after_create_commit :log_creation
  after_update_commit :log_creation
end
