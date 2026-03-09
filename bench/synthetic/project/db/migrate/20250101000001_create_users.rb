class CreateUsers < ActiveRecord::Migration[7.0]
  def change
    # Missing timestamps triggers CreateTableWithTimestamps
    create_table :users do |t|
      t.string :name
      t.string :email
      t.boolean :active
    end

    # DangerousColumnNames: using reserved names
    create_table :records do |t|
      t.string :save
      t.string :class
      t.timestamps
    end

    # ThreeStateBooleanColumn: boolean without null: false + default
    create_table :posts do |t|
      t.boolean :published
      t.boolean :featured, default: false
      t.timestamps
    end

    # NotNullColumn: null: false without default
    add_column :users, :role, :string, null: false

    # AddColumnIndex: add_column with index option
    add_column :users, :department_id, :integer, index: true
  end
end
