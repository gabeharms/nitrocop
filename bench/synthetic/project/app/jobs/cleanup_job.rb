class CleanupJob < ActiveJob::Base
  def perform
    OldRecord.where("created_at < ?", 30.days.ago).delete_all
  end
end

class NotificationJob < ActiveJob::Base
  def perform(user_id)
    User.find(user_id).notify
  end
end

class ExportJob < ActiveJob::Base
  def perform(format)
    Report.generate(format)
  end
end
