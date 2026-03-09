class Modify_tables < ActiveRecord::Migration[7.0]
  def change
    add_column :users, :first_name, :string
    add_column :users, :last_name, :string
    add_column :users, :phone, :string

    execute "UPDATE users SET role = 'member'"
    remove_column :posts, :legacy_field
    change_column :users, :name, :text
  end
end
