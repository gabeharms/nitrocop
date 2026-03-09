class OnlyUp < ActiveRecord::Migration[7.0]
  def up
    add_column :users, :token, :string
  end
end
