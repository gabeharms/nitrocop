ActiveSupport.on_load(:active_record) { include MyClass }
ActiveSupport.on_load(:action_controller) do
  include MyModule
end
foo.extend(MyClass)
MyClass1.prepend(MyClass)
ActiveRecord::Base.include
name.include?('bob')

# RAILS_7_1_LOAD_HOOKS should not fire when TargetRailsVersion < 7.1
ActiveModel::Model.include(MyModule)
ActiveRecord::TestFixtures.prepend(TestFixtures)
ActiveRecord::ConnectionAdapters::PostgreSQLAdapter.prepend(PostgresExtension)
ActiveRecord::ConnectionAdapters::TrilogyAdapter.prepend(SemianAdapter)
