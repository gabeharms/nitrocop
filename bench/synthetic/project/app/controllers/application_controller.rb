require_dependency "concerns/authenticatable"
require_dependency "concerns/authorizable"
require_dependency "concerns/trackable"

class ApplicationController < ActionController::Base
  before_action :set_locale
  before_action :set_timezone

  # IgnoredSkipActionFilterOption
  skip_before_action :login_required, only: :show, if: :trusted_origin?
  skip_after_action :cleanup, only: :index, if: -> { condition }
  skip_before_action :verify_token, except: :admin, if: :internal_request?

  private

  def set_locale
    I18n.locale = params[:locale] || I18n.default_locale
  end

  def set_timezone
    Time.zone = current_user&.timezone || "UTC"
  end

  def local_env?
    Rails.env.development? || Rails.env.test?
  end

  # ActiveSupportAliases
  def check_prefix(str)
    str.starts_with?("pre")
  end

  def check_suffix(str)
    str.ends_with?("post")
  end

  def add_item(list)
    list.append(42)
  end

  # RootJoinChain
  def config_path
    Rails.root.join("config").join("locales")
  end

  def asset_path
    Rails.root.join("app").join("assets")
  end

  def public_logo
    Rails.public_path.join("images").join("logo.png")
  end
end
