before_save :upcase_title
after_destroy :log_deletion
before_update :normalize_data
def custom_method
  super
end
def save
  # No super call
  do_something
end

# Module context: should not flag (not a class inheriting from AR)
module PaymentConcern
  def save
    super
  end
  def destroy
    super
  end
end

# No parent class: should not flag
class Opportunity
  def save
    super
  end
end

# Non-AR parent class: should not flag
class Provider < SomeOtherClass
  def destroy
    super
  end
end

# Explicit super(): should not flag (only bare super triggers)
class Journal < ApplicationRecord
  def save(*args)
    notes_empty? ? false : super()
  end
end

# AR class but using super with explicit args
class Record < ActiveRecord::Base
  def update(*args)
    validate_first
    super(args)
  end
end
