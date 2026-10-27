<?php
	/*	
	*	Goodlayers Html Option File
	*	---------------------------------------------------------------------
	*	This file create the class that help you create the input form element
	*	---------------------------------------------------------------------
	*/	
	
	if( !class_exists('gdlr_core_user_option') ){
		
		class gdlr_core_user_option{

			private $user_fields;

			function __construct( $fields = array() ){
				$this->user_fields = $fields;

				add_action('show_user_profile', array(&$this, 'create_extra_profiles'));
				add_action('edit_user_profile', array(&$this, 'create_extra_profiles'));

				add_action('personal_options_update', array(&$this, 'update_extra_profiles'));
				add_action('edit_user_profile_update', array(&$this, 'update_extra_profiles'));

				// add the script 
				add_action('admin_enqueue_scripts', array(&$this, 'enqueue_profile_script') );
			}

			function enqueue_profile_script( $hook ){
				
				if( $hook == 'profile.php' || $hook == 'user-edit.php' ){
					wp_enqueue_style('wp-mediaelement');
					wp_enqueue_script('wp-mediaelement');

					gdlr_core_html_option::include_script(array(
						'style' => 'html-option-small'
					));		
				}
			}

			function create_extra_profiles( $user ){

				foreach( $this->user_fields as $slug => $field ){
					
					echo '<table class="form-table">';
					echo '<tr>';
					echo '<th><label for="' . esc_attr($slug) . '">' . $field['title'] . '</label></th>';
					echo '<td>';

					switch( $field['type'] ){
						case 'text': 
							echo '<input type="text" name="' . esc_attr($slug) . '" value="' . esc_attr(get_the_author_meta($slug, $user->ID)) . '" />';
							break;
						case 'textarea':
							echo '<textarea name="' . esc_attr($slug) . '" >' . esc_textarea(get_the_author_meta($slug, $user->ID)) . '</textarea>';
							break;
						case 'upload': 
							$field['title'] = '';
							$field['slug'] = $slug;
							$field['value'] = get_the_author_meta($slug, $user->ID);
							$field['with-name'] = true;
							echo gdlr_core_html_option::get_element($field);
							break;

						default: break;
					}
					echo '</td>';
					echo '</tr>';
					echo '</table>';
				}

			}

			function update_extra_profiles( $user_id ){
				if( !current_user_can('edit_user', $user_id) ){
					return false;
				}

				foreach( $this->user_fields as $slug => $field ){
					if( isset($slug) ){
						update_user_meta($user_id, $slug, $_POST[$slug]);
					}
				}

			}
			
		} // gdlr_core_html_option
	
	} // class_exists