require_dependency "concerns/authenticatable"
require_dependency "concerns/authorizable"
require_dependency "concerns/trackable"

class ApplicationController < ActionController::Base
  before_action :set_locale
  before_action :set_timezone

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
end
