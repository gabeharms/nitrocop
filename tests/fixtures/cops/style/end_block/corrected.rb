at_exit { puts 'Goodbye!' }

at_exit { cleanup }

at_exit { save_state }
