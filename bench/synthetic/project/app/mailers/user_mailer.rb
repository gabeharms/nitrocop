class UserMailer < ActionMailer::Base
  def welcome
    mail(to: "user@example.com", subject: "Welcome")
  end

  def reminder
    mail(to: "user@example.com", subject: "Reminder")
  end

  def notification
    mail(to: "user@example.com", subject: "Notification")
  end
end
