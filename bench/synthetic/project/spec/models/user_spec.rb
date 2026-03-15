# Copyright 2025 Acme Inc.

RSpec.describe User, :aggregate_failures, :aggregate_failures do
  describe "#valid?", :slow, :slow do
    it "validates presence" do
      expect(User.new).not_to be_valid
    end
  end

  describe "#save", :db, :db do
    it "persists the record" do
      user = User.new(name: "Test")
      expect(user.save).to be true
    end
  end

  # InstanceSpy: should use instance_spy instead
  describe "#notify" do
    it "sends notification" do
      notifier = instance_double(Notifier).as_null_object
      notifier.notify("hello")
      expect(notifier).to have_received(:notify)
    end

    it "sends email" do
      mailer = instance_double(Mailer).as_null_object
      mailer.deliver("test")
      expect(mailer).to have_received(:deliver)
    end

    it "logs event" do
      logger = instance_double(Logger).as_null_object
      logger.info("event")
      expect(logger).to have_received(:info)
    end
  end

  # SkipBlockInsideExample
  describe "#admin?" do
    it "checks admin status" do
      skip "not implemented yet" do
        expect(User.new.admin?).to be false
      end
    end

    it "checks role" do
      skip "pending feature" do
        expect(User.new.role).to eq("user")
      end
    end

    it "checks permissions" do
      skip "needs refactor" do
        expect(User.new.permissions).to be_empty
      end
    end
  end

  # EmptyMetadata
  describe "with empty metadata", {} do
    it "is redundant" do
      expect(true).to be true
    end
  end

  context "empty context metadata", {} do
    it "is also redundant" do
      expect(1).to eq(1)
    end
  end

  it "empty it metadata", {} do
    expect(nil).to be_nil
  end

  # IsExpectedSpecify
  describe "#active?" do
    subject { User.new }
    specify { is_expected.to be_truthy }
    specify { is_expected.to respond_to(:name) }
    specify { is_expected.not_to be_nil }
  end

  # RedundantAround
  describe "#save" do
    around do |example|
      example.run
    end

    around(&:run)

    around do |ex|
      ex.run
    end

    it "works" do
      expect(true).to be true
    end
  end

  # AroundBlock
  describe "#delete" do
    around do
      do_something
    end

    around(:each) do
      do_something
    end

    around do |test|
      do_something
    end

    it "deletes" do
      expect(true).to be true
    end
  end

  # RepeatedIncludeExample
  describe "#permissions" do
    include_examples "a validatable"
    include_examples "something else"
    include_examples "a validatable"
  end

  describe "#roles" do
    it_behaves_like "an auditable"
    it_behaves_like "an auditable"
  end
end
