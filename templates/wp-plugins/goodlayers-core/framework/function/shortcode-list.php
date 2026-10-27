<?php
	/*	
	*	Goodlayers Shortcode List
	*	---------------------------------------------------------------------
	*	This file register shortcode to wysiwyg editor
	*	---------------------------------------------------------------------
	*/

	add_action('init', 'gdlr_core_register_tinymce_button');
	if( !function_exists('gdlr_core_register_tinymce_button') ){
		function gdlr_core_register_tinymce_button() {
		    add_filter('mce_buttons', 'gdlr_core_add_tinymce_button');
		    add_filter('mce_external_plugins', 'gdlr_core_set_tinymce_button_script');
		}
	}

	if( !function_exists('gdlr_core_add_tinymce_button') ){
		function gdlr_core_add_tinymce_button($buttons){
		   array_push($buttons, 'gdlr_core');
		   return $buttons;
		}
	}

	if( !function_exists('gdlr_core_set_tinymce_button_script') ){
		function gdlr_core_set_tinymce_button_script($plugin_array){
		    $plugin_array['gdlr_core'] = GDLR_CORE_URL . '/framework/js/shortcode-list.js';
		    return $plugin_array;
		}
	}

	add_action('admin_print_scripts', 'gdlr_core_print_shortcodes_variable');
	if( !function_exists('gdlr_core_print_shortcodes_variable') ){
		function gdlr_core_print_shortcodes_variable(){
			$shortcode_list = apply_filters('gdlr_core_shortcode_list', array());
			$count = 0;

			echo '<script>';
			echo 'var gdlr_core_shortcodes = [';
			foreach( $shortcode_list as $shortcode ){
				if( $count > 0 ){
					echo ', ';
				}
				if( !empty($shortcode['title']) && !empty($shortcode['value']) ){
					echo '{ title: \'' . $shortcode['title'] . '\', value: \'' . $shortcode['value'] . '\' }';
				}
				$count++;
			}
			echo '];';
			echo '</script>';
		}
	}