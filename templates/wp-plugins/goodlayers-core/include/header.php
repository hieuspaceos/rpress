<?php
	
	if( is_admin() ){ add_action('after_setup_theme', 'gdlr_core_enable_header_post_type'); }
	if( !function_exists('gdlr_core_enable_header_post_type') ){
		function gdlr_core_enable_header_post_type(){
			$enable_custom_header = apply_filters('gdlr_core_enable_header_post_type', false);

			if( $enable_custom_header ){
				add_action('init', 'gdlr_core_header_post_init');
			}
		}
	}

	if( !function_exists('gdlr_core_header_post_init') ){
		function gdlr_core_header_post_init(){

			$labels = array(
				'name'               => esc_html__('Header', 'goodlayers-core'),
				'singular_name'      => esc_html__('Header', 'goodlayers-core'),
				'menu_name'          => esc_html__('Header', 'goodlayers-core'),
				'name_admin_bar'     => esc_html__('Header', 'goodlayers-core'),
				'add_new'            => esc_html__('Add New', 'goodlayers-core'),
				'add_new_item'       => esc_html__('Add New Header', 'goodlayers-core'),
				'new_item'           => esc_html__('New Header', 'goodlayers-core'),
				'edit_item'          => esc_html__('Edit Header', 'goodlayers-core'),
				'view_item'          => esc_html__('View Header', 'goodlayers-core'),
				'all_items'          => esc_html__('All Header', 'goodlayers-core'),
				'search_items'       => esc_html__('Search Header', 'goodlayers-core'),
				'parent_item_colon'  => esc_html__('Parent Header:', 'goodlayers-core'),
				'not_found'          => esc_html__('No header found.', 'goodlayers-core'),
				'not_found_in_trash' => esc_html__('No header found in Trash.', 'goodlayers-core')
			);

			$args = array(
				'labels'             => $labels,
				'description'        => esc_html__('Description.', 'goodlayers-core'),
				'public'             => false,
				'publicly_queryable' => false,
				'exclude_from_search'=> true,
				'show_ui'            => true,
				'show_in_menu'       => true,
				'query_var'          => true,
				'has_archive'        => false,
				'hierarchical'       => false,
				'menu_position'      => null,
				'supports'           => array('title', 'custom-fields')
			);

			register_post_type('gdlr_core_header', $args);

		}
	}

	// create an option
	if( is_admin() ){ add_action('init', 'gdlr_core_header_option_init'); }
	if( !function_exists('gdlr_core_header_option_init') ){
		function gdlr_core_header_option_init(){

			if( class_exists('gdlr_core_page_option') ){
				global $pagenow;
				$with_default = ($pagenow == 'post-new.php');

				$header_options = apply_filters('gdlr_core_header_options', array(), $with_default);
				new gdlr_core_page_option(array(
					'slug' => 'gdlr-core-header-settings',
					'title' => esc_html__('Settings', 'goodlayers-core'),
					'post_type' => array('gdlr_core_header'),
					'options' => $header_options
				));

				$header_typo_options = apply_filters('gdlr_core_header_typography_options', array(), $with_default);
				if( !empty($header_typo_options) ){
					new gdlr_core_page_option(array(
						'slug' => 'gdlr-core-header-typography',
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'post_type' => array('gdlr_core_header'),
						'options' => $header_typo_options
					));
				}
				

				$header_color_options = apply_filters('gdlr_core_header_color_options', array(), $with_default);
				new gdlr_core_page_option(array(
					'slug' => 'gdlr-core-header-color',
					'title' => esc_html__('Color', 'goodlayers-core'),
					'post_type' => array('gdlr_core_header'),
					'options' => $header_color_options
				));
			}
		}
	}

	if( !function_exists('gdlr_core_prepend_page_id') ){
		function gdlr_core_prepend_page_id( $css ){

			$page_id = '.gdlr-core-page-id';

			$css_array = array_map('trim', explode('}', $css));
			foreach($css_array as $line_key => $line_value){
				$line_value_array = array_map('trim', explode(',', $line_value));

			    foreach($line_value_array as $key => $value ){
			    	if( !empty($value) ){
			    		if( strpos($value, 'body') !== false ){
				        	$line_value_array[$key] = str_replace('body', 'body' . $page_id, $value);
				        }else if( substr_count($value, '{') > 1 ){
				        	$brace_pos = strpos($value, '{');
				        	$line_value_array[$key] = substr($value, 0, $brace_pos+1) . " {$page_id} " . substr($value, $brace_pos+1);
				        }else{
				        	$line_value_array[$key] = $page_id . ' ' . $value;
				        }

				        if( strpos($value, 'rgba') !== false ){
				        	break;
				        }
			    	}
			    }
				
			    $css_array[$line_key] = implode(', ', $line_value_array);
			}

			$css = implode(' } ', $css_array);

			return $css;
		}
	}

	add_action('gdlr_core_after_update_page_option', 'gdlr_core_set_header_option_css');
	if( !function_exists('gdlr_core_set_header_option_css') ){
		function gdlr_core_set_header_option_css( $post_id ){
			$header_css = '';

			$general_options = apply_filters('gdlr_core_header_options', array());
			$general = get_post_meta($post_id, 'gdlr-core-header-settings', true);
			
			foreach( $general_options as $tab ){
				foreach( $tab['options'] as $option_slug => $option ){
					if( empty($option['selector']) ) continue; 
					
					if( !empty($general[$option_slug]) || (isset($general[$option_slug]) && $general[$option_slug] === '0') ){
						$option['selector'] = gdlr_core_prepend_page_id($option['selector']);
						$header_css .= gdlr_core_option_to_css($option_slug, $option, $general);
					}
				}
			}

			$typography_options = apply_filters('gdlr_core_header_typography_options', array());
			$typography = get_post_meta($post_id, 'gdlr-core-header-typography', true);
			
			foreach( $typography_options as $tab ){
				foreach( $tab['options'] as $option_slug => $option ){
					if( empty($option['selector']) ) continue; 
					
					if( !empty($typography[$option_slug]) || (isset($typography[$option_slug]) && $typography[$option_slug] === '0') ){
						$option['selector'] = gdlr_core_prepend_page_id($option['selector']);
						$header_css .= gdlr_core_option_to_css($option_slug, $option, $typography);
					}
				}
			}

			$color_options = apply_filters('gdlr_core_header_color_options', array());
			$color = get_post_meta($post_id, 'gdlr-core-header-color', true);

			foreach( $color_options as $tab ){
				foreach( $tab['options'] as $option_slug => $option ){
					if( empty($option['selector']) ) continue; 

					if( !empty($color[$option_slug]) || (isset($color[$option_slug]) && $color[$option_slug] === '0') ){
						$option['selector'] = gdlr_core_prepend_page_id($option['selector']);
						$header_css .= gdlr_core_option_to_css($option_slug, $option, $color);
					}
				}
			}

			update_post_meta($post_id, 'gdlr-core-custom-header-css', $header_css);

		}
	}