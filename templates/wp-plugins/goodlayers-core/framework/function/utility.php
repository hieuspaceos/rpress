<?php
	/*	
	*	Goodlayers Utility Files
	*	---------------------------------------------------------------------
	*	This file contains the function that helps doing things
	*	---------------------------------------------------------------------
	*/

	if( !function_exists('gdlr_core_sanitize_input') ){
		function gdlr_core_sanitize_input($fields){

			if( !current_user_can('unfiltered_html') ){ 
				if( !empty($fields) ){
					foreach( $fields as $key => $value ){
						if( is_array($value) ){
							$fields[$key] = gdlr_core_sanitize_input($value);
						}else{
							$fields[$key] = wp_kses_post($value);
						}
					}
				}
			}

			return $fields;
		}
	}
	if( !function_exists('gdlr_core_sanitize_script') ){
		function gdlr_core_sanitize_script($fields){

			if( !empty($fields) ){
				foreach( $fields as $key => $value ){
					if( is_array($value) ){
						$fields[$key] = gdlr_core_sanitize_script($value);
					}else{
						$fields[$key] = esc_html($value);
					}
				}
			}

			return $fields;
		}
	}


	// get option for uses
	if( !function_exists('gdlr_core_get_option') ){
		function gdlr_core_get_option($option, $key = '', $default = ''){
			$option = 'gdlr_core_admin_option_' . $option;
			
			if( empty($GLOBALS[$option]) ){
				$GLOBALS[$option] = get_option($option, '');
			}
				
			if( !empty($key) ){
				if( !empty($GLOBALS[$option][$key]) || (isset($GLOBALS[$option][$key]) && $GLOBALS[$option][$key] === '0') ){
					return $GLOBALS[$option][$key];
				}else{
					return $default;
				}
			}else{
				return $GLOBALS[$option];
			}
		}
	}

	if( !function_exists('gdlr_core_array_insert') ){
		function gdlr_core_array_insert($array, $position, $insert){

		    if( !is_int($position) ){
		        $position = array_search($position, array_keys($array)) + 1;
		    }
		    
		    return array_slice($array, 0, $position) + $insert + array_slice($array, $position);
		}
	}

	// Setup a post object and store the original loop item so we can reset it later
	if( !function_exists('gdlr_core_setup_admin_postdata') ){
		function gdlr_core_setup_admin_postdata(){
			global $post;

			if( is_admin() ){
				global $gdlr_core_post;
				$gdlr_core_post = $post;
			}
		}
	}

	// Reset $post back to the original item
	if( !function_exists('gdlr_core_reset_admin_postdata') ){
		function gdlr_core_reset_admin_postdata(){
			global $gdlr_core_post;

			if( is_admin() && !empty($gdlr_core_post) ){
				global $post;
				$post = $gdlr_core_post;
				setup_postdata($post);

				// clean up the data
				unset($gdlr_core_post);
			}
		}
	}

	// include utility function for uses 
	// make sure to call this function inside wp_enqueue_script action
	if( !function_exists('gdlr_core_include_utility_script') ){
		function gdlr_core_include_utility_script(){
			
			if( is_admin() ){
				wp_enqueue_style('google-Montserrat', '//fonts.googleapis.com/css?family=Montserrat:400,700');
			}
		
			gdlr_core_include_icon_font('font-awesome');
			gdlr_core_include_icon_font('elegant-font');
						
			wp_enqueue_style('gdlr-core-utility', GDLR_CORE_URL . '/framework/css/utility.css');
			
			wp_enqueue_script('gdlr-core-utility', GDLR_CORE_URL . '/framework/js/utility.js', array('jquery'), false, true);
			wp_localize_script('gdlr-core-utility', 'gdlr_utility', array(
				'confirm_head' => esc_html__('Just to confirm', 'goodlayers-core'),
				'confirm_text' => esc_html__('Are you sure to do this ?', 'goodlayers-core'),
				'confirm_sub' => esc_html__('* Please noted that this could not be undone.', 'goodlayers-core'),
				'confirm_yes' => esc_html__('Yes', 'goodlayers-core'),
				'confirm_no' => esc_html__('No', 'goodlayers-core'),
			));
			
		}
	}	
	
	// change any string to valid html id
	if( !function_exists('gdlr_core_string_to_slug') ){
		function gdlr_core_string_to_slug( $string ){
			// lower case everything
			$string = strtolower($string);
			
			// make alphanumeric (removes all other characters)
			$string = preg_replace("/[^a-z0-9_\s-]/", "", $string);
			
			// clean up multiple dashes or whitespaces
			$string = preg_replace("/[\s\-_]+/", " ", $string);
			
			// remove space at the front and back
			$string = trim($string);
			
			// convert whitespaces and underscore to dash
			$string = preg_replace("/[\s_]/", "-", $string);
    
			return $string;
		}
	}
	
	// get all thumbnail name
	if( !function_exists('gdlr_core_get_thumbnail_list') ){
		function gdlr_core_get_thumbnail_list(){
			$ret = array();
			
			$thumbnails = get_intermediate_image_sizes();
			$ret['full'] = esc_html__('full size', 'goodlayers-core');
			foreach( $thumbnails as $thumbnail ) {
				if( !empty($GLOBALS['_wp_additional_image_sizes'][$thumbnail]) ){
					$width = $GLOBALS['_wp_additional_image_sizes'][$thumbnail]['width'];
					$height = $GLOBALS['_wp_additional_image_sizes'][$thumbnail]['height'];
				}else{
					$width = get_option($thumbnail . '_size_w', '');
					$height = get_option($thumbnail . '_size_h', '');
				}
				$ret[$thumbnail] = $thumbnail . ' ' . $width . '-' . $height;
			}
			return $ret;
		}
	}
	if( !function_exists('gdlr_core_get_video_size') ){
		function gdlr_core_get_video_size( $size = '' ){

			if( empty($size) || $size == 'full' ){
				return array( 'width'=>640, 'height'=>360 );
			}
			
			if( is_array($size) && ( $size['width'] == '100%' || $size['height'] == '100%') ){
				return array( 'width'=>640, 'height'=>360 );
			}
			
			if( !empty($GLOBALS['_wp_additional_image_sizes'][$size]) ){
				$width = $GLOBALS['_wp_additional_image_sizes'][$size]['width'];
				$height = $GLOBALS['_wp_additional_image_sizes'][$size]['height'];
				if( !empty($width) && !empty($height) ){
					return array( 'width'=>$width, 'height'=>$height );
				}
			}

			return array( 'width'=>640, 'height'=>360 );
		}
	}
	
	// gdlr esc size
	if( !function_exists('gdlr_core_esc_style') ){
		function gdlr_core_esc_style($atts, $wrap = true, $important = false){
			if( empty($atts) || !is_array($atts) ) return '';
			$att_style = '';

			// special attribute
			if( !empty($atts['background-shadow-color']) ){
				if( !empty($atts['background-shadow-size']) ){
					if( is_array($atts['background-shadow-size']) ){
						$bgs_sizex = empty($atts['background-shadow-size']['x'])? '0': $atts['background-shadow-size']['x'];
						$bgs_sizey = empty($atts['background-shadow-size']['y'])? '0': $atts['background-shadow-size']['y'];
						$bgs  = $bgs_sizex . ' ' . $bgs_sizey . ' ' . $atts['background-shadow-size']['size'] . ' ';
					}else{
						$bgs  = $atts['background-shadow-size'] . ' ';
					}

					if( !empty($bgs) ){
						$atts['background-shadow-opacity'] = empty($atts['background-shadow-opacity'])? 0: $atts['background-shadow-opacity'];
						$bgs .= 'rgba(' . gdlr_core_format_datatype($atts['background-shadow-color'], 'rgba') . ',' . $atts['background-shadow-opacity'] . ')';

						$att_style .= 'box-shadow: ' . $bgs . ' ' . ($important? ' !important': '') . '; ';
						$att_style .= '-moz-box-shadow: ' . $bgs . ' ' . ($important? ' !important': '') . '; ';
						$att_style .= '-webkit-box-shadow: ' . $bgs . ' ' . ($important? ' !important': '') . '; ';
					}
				}
			}
			unset($atts['background-shadow-color']);
			unset($atts['background-shadow-size']);
			unset($atts['background-shadow-opacity']);

			if( !empty($atts['skewx']) ){
				$att_style .= 'transform: skewX(' . $atts['skewx'] . 'deg);';
				$att_style .= '-webkit-transform: skewX(' . $atts['skewx'] . 'deg);';
			}
			unset($atts['skewx']);

			// normal attribute
			foreach($atts as $key => $value){
				if( empty($value) ) continue;
				
				switch($key){
					
					case 'blur': 
						if( !empty($value) ){
							$att_style .= "-webkit-filter: blur({$value});";
							$att_style .= "-moz-filter: blur({$value});";
							$att_style .= "-o-filter: blur({$value});";
							$att_style .= "-ms-filter: blur({$value});";
							$att_style .= "filter: blur({$value});";
						}
						break;

					case 'border-radius': 
						if( is_array($value) ){
							if( !empty($value['top']) || !empty($value['right']) || !empty($value['bottom']) || !empty($value['left']) ){
								$value = wp_parse_args($value, array('top'=>'0', 'right'=>'0', 'bottom'=>'0', 'left'=>'0'));
								$value = "{$value['top']} {$value['right']} {$value['bottom']} {$value['left']}";
							}else if( !empty($value['top-left']) || !empty($value['top-right']) || !empty($value['bottom-left']) || !empty($value['bottom-right']) ){
								$value = wp_parse_args($value, array('top-left'=>'0', 'top-right'=>'0', 'bottom-left'=>'0', 'bottom-right'=>'0'));
								$value = "{$value['top-left']} {$value['top-right']} {$value['bottom-right']} {$value['bottom-left']}";
							}else if( !empty($value['t-left']) || !empty($value['t-right']) || !empty($value['b-left']) || !empty($value['b-right']) ){
								$value = wp_parse_args($value, array('top-left'=>'0', 'top-right'=>'0', 'bottom-left'=>'0', 'bottom-right'=>'0'));
								$value = "{$value['t-left']} {$value['t-right']} {$value['b-right']} {$value['b-left']}";
							}else{
								$value = '';
							}
						}

						if( !empty($value) ){
							$att_style .= "border-radius: {$value};";
							$att_style .= "-moz-border-radius: {$value};";
							$att_style .= "-webkit-border-radius: {$value};";
						}
						break;
					
					case 'gradient': 
					case 'gradient-v': 
						if( is_array($value) && sizeOf($value) > 1 ){
							$atts = '';
							if( $key == 'gradient-v' ){
								$atts = 'to right, ';
							}

							if( is_array($value[0]) ){
								$rgba_value = gdlr_core_format_datatype($value[0][0], 'rgba');
								$color1 = "rgba({$rgba_value}, {$value[0][1]})";
							}else{
								$color1 = $value[0];
							}
							if( is_array($value[1]) ){
								$rgba_value = gdlr_core_format_datatype($value[1][0], 'rgba');
								$color2 = "rgba({$rgba_value}, {$value[1][1]})";
							}else{
								$color2 = $value[1];
							}

							if( empty($value[2]) ){
								$att_style .= "background: linear-gradient({$atts}{$color1}, {$color2});";
								$att_style .= "-moz-background: linear-gradient({$atts}{$color1}, {$color2});";
								$att_style .= "-o-background: linear-gradient({$atts}{$color1}, {$color2});";
								$att_style .= "-webkit-background: linear-gradient({$atts}{$color1}, {$color2});";
							}else{
								if( is_array($value[2]) ){
									$rgba_value = gdlr_core_format_datatype($value[2][0], 'rgba');
									$color3 = "rgba({$rgba_value}, {$value[2][1]})";
								}else{
									$color3 = $value[2];
								}

								$att_style .= "background: linear-gradient({$atts}{$color1}, {$color2}, {$color3});";
								$att_style .= "-moz-background: linear-gradient({$atts}{$color1}, {$color2}, {$color3});";
								$att_style .= "-o-background: linear-gradient({$atts}{$color1}, {$color2}, {$color3});";
								$att_style .= "-webkit-background: linear-gradient({$atts}{$color1}, {$color2}, {$color3});";
							}
							
						}
						break;
					
					case 'background':
					case 'background-color':
						if( is_array($value) ){
							$rgba_value = gdlr_core_format_datatype($value[0], 'rgba');
							$att_style .= "{$key}: rgba({$rgba_value}, {$value[1]}) " . ($important? ' !important': '') . ";";
						}else{
							$att_style .= "{$key}: {$value} " . ($important? ' !important': '') . ";";
						}
						break;

					case 'background-image':
						if( $value == 'none' ){
							$att_style .= "background-image: none;";
						}else if( is_numeric($value) ){
							$image_url = gdlr_core_get_image_url($value);
							if( !empty($image_url) ){
								$att_style .= "background-image: url({$image_url}) " . ($important? ' !important': '') . ";";
							}
						}else{
							$att_style .= "background-image: url({$value}) " . ($important? ' !important': '') . ";";
						}
						break;
					
					case 'padding':
					case 'margin':
					case 'border-width':
						if( is_array($value) ){
							if( !empty($value['top']) && !empty($value['right']) && !empty($value['bottom']) && !empty($value['left']) ){
								$att_style .= "{$key}: {$value['top']} {$value['right']} {$value['bottom']} {$value['left']}" . ($important? ' !important': '') . ";";
							}else{
								foreach($value as $pos => $val){
									if( $pos != 'settings' && (!empty($val) || $val === '0') ){
										if( $key == 'border-width' ){
											$att_style .= "border-{$pos}-width: {$val}" . ($important? ' !important': '') . ";";
										}else{
											$att_style .= "{$key}-{$pos}: {$val}" . ($important? ' !important': '') . ";";
										}
									}
								}
							}
						}else{
							$att_style .= "{$key}: {$value};";
						}
						break;
					
					default: 
						$value = is_array($value)? ((empty($value[0]) || $value[0] === '0')? '': $value[0]): $value;
						$att_style .= "{$key}: {$value} " . ($important? ' !important': '') . ";";
				}
			}
			
			if( !empty($att_style) ){
				if( $wrap ){
					return 'style="' . esc_attr($att_style) . '" ';
				}
				return $att_style;
			}
			return '';
		}
	}
	
	// process data sent from the post variable
	if( !function_exists('gdlr_core_process_post_data') ){
		function gdlr_core_process_post_data( $post ){
			return stripslashes_deep($post);
		}
	}		
	
	// format data to specific type
	if( !function_exists('gdlr_core_format_datatype') ){
		function gdlr_core_format_datatype( $value, $data_type ){
			if( $data_type == 'color' ){
				if( $value == 'transparent' ){
					return $value;
				}
				return (strpos($value, '#') === false)? '#' . $value: $value; 
			}else if( $data_type == 'rgba' ){
				$value = str_replace('#', '', $value);
				if(strlen($value) == 3) {
					$r = hexdec(substr($value,0,1) . substr($value,0,1));
					$g = hexdec(substr($value,1,1) . substr($value,1,1));
					$b = hexdec(substr($value,2,1) . substr($value,2,1));
				}else{
					$r = hexdec(substr($value,0,2));
					$g = hexdec(substr($value,2,2));
					$b = hexdec(substr($value,4,2));
				}
				return $r . ', ' . $g . ', ' . $b;
			}else if( $data_type == 'text' ){
				return trim($value);
			}else if( $data_type == 'pixel' ){
				return (is_numeric($value))? $value . 'px': $value;
			}else if( $data_type == 'file' ){
				if(is_numeric($value)){
					$image_src = wp_get_attachment_image_src($value, 'full');	
					return (!empty($image_src))? $image_src[0]: false;
				}else{
					return $value;
				}
			}else if( $data_type == 'font'){
				return trim($value);
			}else if( $data_type == 'percent' ){
				return (is_numeric($value))? $value . '%': $value;
			}else if( $data_type == 'opacity' ){
				return (intval($value) / 100);
			} 
		}
	}	
	
	// retrieve all categories from each post type
	if( !function_exists('gdlr_core_get_taxonomies') ){	
		function gdlr_core_get_taxonomies(){

			$taxonomy = get_taxonomies();
			unset($taxonomy['nav_menu']);
			unset($taxonomy['link_category']);
			unset($taxonomy['post_format']);

			return $taxonomy;

		}
	}

	// retrieve all categories from each post type
	if( !function_exists('gdlr_core_get_term_list') ){	
		function gdlr_core_get_term_list( $taxonomy, $cat = '', $with_all = false ){
			$term_atts = array(
				'taxonomy'=>$taxonomy, 
				'hide_empty'=>0,
				'number'=>999
			);
			if( !empty($cat) ){
				if( is_array($cat) ){
					$term_atts['slug'] = $cat;
				}else{
					$term_atts['parent'] = $cat;
				}
			}
			$term_list = get_categories($term_atts);

			$ret = array();
			if( !empty($with_all) ){
				$ret[$cat] = esc_html__('All', 'goodlayers-core'); 
			}

			if( !empty($term_list) ){
				foreach( $term_list as $term ){
					if( !empty($term->slug) && !empty($term->name) ){
						$ret[$term->slug] = $term->name;
					}
				}
			}

			return $ret;
		}	
	}
	if( !function_exists('gdlr_core_get_term_list_id') ){	
		function gdlr_core_get_term_list_id( $taxonomy ){
			$term_atts = array(
				'taxonomy'=>$taxonomy, 
				'hide_empty'=>0,
				'number'=>5000
			);

			$term_list = get_categories($term_atts);

			$ret = array();
			if( !empty($term_list) ){
				foreach( $term_list as $term ){
					if( !empty($term->term_id) && !empty($term->name) ){
						$ret[$term->term_id] = $term->name;
					}
				}
			}

			return $ret;
		}	
	}
	
	// user role list
	if( !function_exists('gdlr_core_get_user_role_list') ){	
		function gdlr_core_get_user_role_list(){
			global $wp_roles; return $wp_roles->get_names();
		}
	}

	// retrieve all menus
	if( !function_exists('gdlr_core_get_menu_list') ){	
		function gdlr_core_get_menu_list($with_default = true){
			$menus = wp_get_nav_menus();
			$list = array();
			if( $with_default ){
				$list[''] = esc_html__('Default', 'goodalyers-core');
			} 
			
			foreach($menus as $menu){
				$list[$menu->term_id] = $menu->name;
			}

			return $list;
		}
	}

	// retrieve all posts from each post type
	if( !function_exists('gdlr_core_get_post_list') ){	
		function gdlr_core_get_post_list( $post_type, $type = 'id' ){
			$post_list_number = apply_filters('gdlr_core_post_list_amount', 999);
			$post_list = get_posts(array('post_type' => $post_type, 'numberposts'=>$post_list_number));

			$ret = array();
			if( !empty($post_list) ){
				foreach( $post_list as $post ){
					if( $type == 'slug' ){
						$ret[$post->post_name] = $post->post_title;
					}else{
						$ret[$post->ID] = $post->post_title;
					}
				}
			}
				
			return $ret;
		}	
	}

	// retrieve all user
	if( !function_exists('gdlr_core_get_author_list') ){	
		function gdlr_core_get_author_list( $roles = array() ){

			$args = empty($roles)? array(): array('role__in' => $roles);
			$user_list = get_users($args);
		
			$ret = array();
			if( !empty($user_list) ){
				foreach($user_list as $user){
					$ret[$user->data->ID] = $user->data->user_nicename;
				}
			}

			return $ret;
		}	
	}

	// page builder content/text filer to execute the shortcode	
	if( !function_exists('gdlr_core_content_filter') ){
		add_filter( 'gdlr_core_the_content', 'wptexturize'        ); add_filter( 'gdlr_core_the_content', 'convert_smilies'    );
		add_filter( 'gdlr_core_the_content', 'convert_chars'      ); add_filter( 'gdlr_core_the_content', 'prepend_attachment' );	
		add_filter( 'gdlr_core_the_content', 'wpautop');
		add_filter( 'gdlr_core_the_content', 'shortcode_unautop');
		add_filter( 'gdlr_core_the_content', 'gdlr_core_do_shortcode', 11 );
		function gdlr_core_content_filter( $content, $main_content = false ){
			if($main_content) return str_replace( ']]>', ']]&gt;', apply_filters('the_content', $content) );
			return apply_filters('gdlr_core_the_content', $content);
		}		
	}
	
	if( !function_exists('gdlr_core_content_oembed_filter') ){
		add_filter( 'gdlr_core_the_content', 'gdlr_core_content_oembed_filter', 9);
		function gdlr_core_content_oembed_filter( $content ){
			return preg_replace_callback( '|("?https?://[^\s"<]+)|im', 'gdlr_core_content_oembed', $content );
		}
	}
	if( !function_exists('gdlr_core_content_oembed') ){
		function gdlr_core_content_oembed( $link ){

			if( substr($link[1], 0, 1) == '"' ){ 
				return $link[1]; 
			}

			if( preg_match('/youtube|youtu\.be|vimeo|spotify/', $link[1]) ){
				$html = wp_oembed_get($link[1]);
				
				if( $html ) return $html;
			}
			return $link[1];
		}
	}
	add_filter('oembed_dataparse', 'gdlr_core_oembed_cookies');
	add_filter('embed_oembed_html', 'gdlr_core_oembed_cookies');
	if( !function_exists('gdlr_core_oembed_cookies') ){
		function gdlr_core_oembed_cookies( $return ){
			$youtube_cookies = gdlr_core_youtube_cookies();

			if( !$youtube_cookies ){
				if( strpos($return, 'youtube-nocookie') === false ){
					$return = str_replace('youtube', 'youtube-nocookie', $return);
				}
			}
			return $return;
		}
	}


	if( !function_exists('gdlr_core_remove_extra_p') ){
		function gdlr_core_remove_extra_p( $text ){
			return str_replace('<p></p>', '', $text);
		}
	}
	if( !function_exists('gdlr_core_text_filter') ){
		add_filter('gdlr_core_text_filter', 'do_shortcode', 11);
		function gdlr_core_text_filter( $text ){
			return apply_filters('gdlr_core_text_filter', $text);
		}
	}

	// check broken html tag
	if( is_admin() ){ 
		add_filter( 'gdlr_core_the_content', 'gdlr_core_content_validate'); 
		add_filter( 'gdlr_core_text_filter', 'gdlr_core_content_validate'); 
	}
	if( !function_exists('gdlr_core_content_validate') ){
		function gdlr_core_content_validate( $content ){
			$open_tag  = substr_count($content, '<div');
			$open_tag += substr_count($content, '<iframe');

			$close_tag  = substr_count($content, '</div');
			$close_tag += substr_count($content, '</iframe');
			
			if( $open_tag == $close_tag ){
				return $content;
			}else{
				return esc_html__('Please ensure that all html is opened and closed properly.', 'goodlayers-core');
			}
		}
	}

	// only apply goodlayers shortcode in admin
	if( !function_exists('gdlr_core_do_shortcode') ){
		function gdlr_core_do_shortcode( $content ){

			if( !is_admin() ){
				return do_shortcode($content);
			}else{
				global $shortcode_tags, $gdlr_core_shortcode_tags;

				$allow_tags = apply_filters('gdlr_core_pb_allowed_shortcode', array());

				if( empty($gdlr_core_shortcode_tags) ){
					$gdlr_core_shortcode_tags = array();
					foreach( $shortcode_tags as $tag => $function ){
						if( strpos($tag, 'gdlr_core') !== false || in_array($tag, $allow_tags) ){
							$gdlr_core_shortcode_tags[$tag] = $function;
						}
					}
				}

				$orig_shortcode_tags = $shortcode_tags;

				$shortcode_tags = $gdlr_core_shortcode_tags;

				$content = do_shortcode($content);

				$shortcode_tags = $orig_shortcode_tags;

				return $content;
			}
		}
	}	
	
	// escape content with html
	if( !function_exists('gdlr_core_escape_content') ){
		function gdlr_core_escape_content( $content ){
			return apply_filters('gdlr_core_escape_content', $content);
		}
	}	
	
	// allow specific upload file format
	add_filter('upload_mimes', 'gdlr_core_custom_upload_mimes');
	if( !function_exists('gdlr_core_custom_upload_mimes') ){
		function gdlr_core_custom_upload_mimes( $existing_mimes = array() ){
			if( current_user_can('unfiltered_html') ){
				$existing_mimes['ttf'] = 'application/x-font-ttf';
				$existing_mimes['otf'] = 'application/x-font-opentype'; 
				$existing_mimes['eot'] = 'application/vnd.ms-fontobject'; 
				$existing_mimes['woff'] = 'application/font-woff'; 
				$existing_mimes['svg'] = 'image/svg+xml'; 
			}

			return $existing_mimes;
		}
	}

	// change the object to string
	if( !function_exists('gdlr_core_debug_object') ){
		function gdlr_core_debug_object( $object ){

			ob_start();
			print_r($object);
			$ret = ob_get_contents() . '<br><br>';
			ob_end_clean();

			return $ret;
		}
	}

	// create pagination link
	if( !function_exists('gdlr_core_get_pagination') ){	
		function gdlr_core_get_pagination($max_num_page, $settings = array(), $extra_class = '', $style = ''){
			if( $max_num_page <= 1 ) return '';
		
			$big = 999999999; // need an unlikely integer

			if( empty($settings['pagination-style']) || $settings['pagination-style'] == 'default' ){
				$style = apply_filters('gdlr_core_pagination_style', 'round');
			}else{
				$style = $settings['pagination-style'];
			}
			if( empty($settings['pagination-align']) || $settings['pagination-align'] == 'default' ){
				$align = apply_filters('gdlr_core_pagination_align', 'right');
			}else{
				$align = $settings['pagination-align'];
			}

			$with_border = (strpos($style, '-border') !== false);
			$style = str_replace('-border', '', $style);
			$current_page = empty($settings['paged']) ? 1: $settings['paged'];

			$pagination_class  = ' gdlr-core-style-' .  $style;
			$pagination_class .= ' gdlr-core-' .  $align . '-align';
			$pagination_class .= empty($with_border)? '': ' gdlr-core-with-border';
			$pagination_class .= empty($extra_class)? '': ' ' . $extra_class;
			

			if( is_single() ){
				$paged_query = empty($settings['paged_query'])? 'page': $settings['paged_query'];

				return '<div class="gdlr-core-pagination ' . esc_attr($pagination_class) . '" ' . gdlr_core_esc_style(array(
					'margin-top' => empty($settings['pagination-top-margin'])? '': $settings['pagination-top-margin']
				)) . ' >' . paginate_links(array(
					'base' => add_query_arg(array($paged_query=>'%#%'), get_permalink()),
					'format' => '?' . $paged_query . '=%#%',
					'current' => max(1, $current_page),
					'total' => $max_num_page,
					'prev_text'=> '',
					'next_text'=> '',
					'show_all' => ($max_num_page <= 5)? true: false
				)) . '</div>';
			}else{
				return '<div class="gdlr-core-pagination ' . esc_attr($pagination_class) . '" ' . gdlr_core_esc_style(array(
					'margin-top' => empty($settings['pagination-top-margin'])? '': $settings['pagination-top-margin']
				)) . ' >' . paginate_links(array(
					'base' => str_replace($big, '%#%', get_pagenum_link($big, false)),
					'format' => '?paged=%#%',
					'current' => max(1, $current_page),
					'total' => $max_num_page,
					'prev_text'=> '',
					'next_text'=> '',
					'show_all' => ($max_num_page <= 5)? true: false
				)) . '</div>';
			}
		}	
	}		
	if( !function_exists('gdlr_core_get_ajax_pagination') ){	
		function gdlr_core_get_ajax_pagination($post_type, $settings, $max_num_page, $target, $extra_class = ''){
			if( $max_num_page <= 1 ) return '';
			
			if( empty($settings['pagination-style']) || $settings['pagination-style'] == 'default' ){
				$style = apply_filters('gdlr_core_pagination_style', 'round');
			}else{
				$style = $settings['pagination-style'];
			}
			if( empty($settings['pagination-align']) || $settings['pagination-align'] == 'default' ){
				$align = apply_filters('gdlr_core_pagination_align', 'right');
			}else{
				$align = $settings['pagination-align'];
			}
			$with_border = (strpos($style, '-border') !== false);
			$style = str_replace('-border', '', $style);
			$current_page = empty($settings['paged']) ? 1: $settings['paged'];

			$pagination_class  = ' gdlr-core-style-' .  $style;
			$pagination_class .= ' gdlr-core-' .  $align . '-align';
			$pagination_class .= empty($with_border)? '': ' gdlr-core-with-border';
			$pagination_class .= empty($extra_class)? '': ' ' . $extra_class;

			$ret  = '<div class="gdlr-core-pagination gdlr-core-js ' . esc_attr($pagination_class) . '" ';
			$ret .= 'data-ajax="gdlr_core_' . esc_attr($post_type) . '_ajax" ';
			$ret .= 'data-settings="' . esc_attr(json_encode($settings)) . '" ';
			$ret .= 'data-target="' . esc_attr($target) . '" ';
			$ret .= 'data-target-action="replace" ';
			$ret .= gdlr_core_esc_style(array(
				'margin-top' => empty($settings['pagination-top-margin'])? '': $settings['pagination-top-margin']
			));
			$ret .= '>';
			for($i=1; $i<=$max_num_page; $i++){
				if( $i == $current_page ){
					$ret .= '<a class="page-numbers gdlr-core-active" data-ajax-name="paged" data-ajax-value="' . $i . '" >' . $i . '</a> ';
				}else{
					$ret .= '<a class="page-numbers" data-ajax-name="paged" data-ajax-value="' . $i . '" >' . $i . '</a> ';
				}
			}
			$ret .= '</div>';

			return $ret;
		}	
	}
	if( !function_exists('gdlr_core_get_ajax_load_more') ){	
		function gdlr_core_get_ajax_load_more($post_type, $settings, $paged, $max_num_page, $target, $extra_class){

			$ret  = '';
			if( $paged <= $max_num_page ){
				$ret  = '<div class="gdlr-core-load-more-wrap gdlr-core-js gdlr-core-center-align ' . esc_attr($extra_class) . '" ';
				$ret .= 'data-ajax="gdlr_core_' . esc_attr($post_type) . '_ajax" ';
				$ret .= 'data-settings="' . esc_attr(json_encode($settings)) . '" ';
				$ret .= 'data-target="' . esc_attr($target) . '" ';
				$ret .= 'data-target-action="append" ';
				$ret .= '>';
				if( $paged <= $max_num_page ){
					$ret .= '<a href="#" class="gdlr-core-load-more gdlr-core-button-color" data-ajax-name="paged" data-ajax-value="' . esc_attr($paged) . '" ' . gdlr_core_esc_style(array(
						'margin-top' => empty($settings['pagination-top-margin'])? '': $settings['pagination-top-margin']
					)) . ' >';
					$ret .= apply_filters('gdlr_core_ajax_load_more_text', esc_html__('Load More', 'goodlayers-core'), $post_type);
					$ret .= '</a>';
				}
				$ret .= '</div>';
			}

			return $ret;
		}
	}
	if( !function_exists('gdlr_core_get_ajax_filterer') ){	
		function gdlr_core_get_ajax_filterer($post_type, $taxonomy, $settings, $target, $extra_class, $filterer_atts = array()){

			$ret  = '<div class="gdlr-core-filterer-wrap gdlr-core-js ' . esc_attr($extra_class) . '" ';
			$ret .= 'data-ajax="gdlr_core_' . esc_attr($post_type) . '_ajax" ';
			$ret .= 'data-settings="' . esc_attr(json_encode($settings)) . '" ';
			$ret .= 'data-target="' . esc_attr($target) . '" ';
			$ret .= 'data-target-action="replace" ';
			$filterer_atts = apply_filters('gdlr_core_filterer_css_atts', $filterer_atts, $settings);
			if( !empty($settings['filterer-bottom-margin']) ){
				$filterer_atts['margin-bottom'] = $settings['filterer-bottom-margin'];
			}
			if( !empty($settings['filterer-top-margin']) ){
				$filterer_atts['margin-top'] = $settings['filterer-top-margin'];
			}
			if( !empty($filterer_atts) ){
				$ret .= gdlr_core_esc_style($filterer_atts);
			}
			$ret .= ' >';

			// for all
			if( empty($settings['category']) ){

				$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-button-color gdlr-core-active" >' . esc_html__('All', 'goodlayers-core') . '</a>';
				$filters = gdlr_core_get_term_list($taxonomy);

				if( $taxonomy == 'product_cat' ){
					unset($filters['uncategorized']);
				}

			// parent category
			}else if( sizeof($settings['category']) == 1 ){

				$term = get_term_by('slug', $settings['category'][0], $taxonomy);
				$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-button-color gdlr-core-active" >' . gdlr_core_escape_content($term->name) . '</a>';
				$filters = gdlr_core_get_term_list($taxonomy, $term->term_id);

			// multiple category select
			}else{

				$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-button-color gdlr-core-active" >' . esc_html__('All', 'goodlayers-core') . '</a>';
				$filters = gdlr_core_get_term_list($taxonomy, $settings['category']);
				
			}

			$filter_sep = apply_filters('gdlr_core_filterer_separator', '');
			foreach( $filters as $slug => $name ){
				$ret .= $filter_sep;
				$ret .= '<a href="#" class="gdlr-core-filterer gdlr-core-button-color" data-ajax-name="category" data-ajax-value="' . esc_attr($slug) . '" >';
				$ret .= gdlr_core_escape_content($name);
				$ret .= '</a>';
			}

			if( !empty($settings['filterer-slide-bar']) ){
				$ret .= '<div class="gdlr-core-filterer-slide-bar" ></div>';
			}
			$ret .= '</div>'; // gdlr-core-filterer-wrap

			return $ret;
		}
	}

	// for preparing srcset
	if( !function_exists('gdlr_core_set_container') ){
		function gdlr_core_set_container( $container = true ){
			global $content_width, $gdlr_core_container, $gdlr_core_container_multiplier, $gdlr_core_item_multiplier;

			if( empty($container) ){
				$gdlr_core_container = 2560;
			}else if( $container === true ){
				$gdlr_core_container = $content_width;
			}else{
				$gdlr_core_container = $container;
			}
			$gdlr_core_content_width = $gdlr_core_container;
			$gdlr_core_container_multiplier = $gdlr_core_item_multiplier = 1;
		}
	}
	// main is column
	if( !function_exists('gdlr_core_set_container_multiplier') ){
		function gdlr_core_set_container_multiplier( $multiplier, $main = true ){
			global $gdlr_core_container, $gdlr_core_container_multiplier, $gdlr_core_item_multiplier;

			if( empty($gdlr_core_container) ){
				gdlr_core_set_container();
			}

			if( $main ){
				$gdlr_core_container_multiplier = $multiplier;
			}else{
				$gdlr_core_item_multiplier = $multiplier;
			}
		}
	}
	if( !function_exists('gdlr_core_get_image_srcset') ){
		function gdlr_core_get_image_srcset( $image_id, $image ){
			
			$enable_srcset = apply_filters('gdlr_core_enable_srcset', true);
			if( !$enable_srcset ) return;
			
			if( empty($image) || empty($image[0]) || empty($image[1]) || empty($image[2]) ) return;
			
			$srcset = '';
			
			// crop image
			$smallest_image = $image;
			$cropped_sizes = array(400, 600, 800);
			foreach( $cropped_sizes as $cropped_size ){
				if( $image[1] > $cropped_size + 100 ){
					$new_height = intval($cropped_size * intval($image[2]) / intval($image[1]));
					$cropped_image = gdlr_core_get_cropped_image( $image_id, $cropped_size, $new_height, false);
					
					if( !empty($cropped_image) ){
						$srcset .= empty($srcset)? '': ', ';
						if( false ){
							$srcset .= rocket_cdn_file($cropped_image) . ' ' . $cropped_size . 'w';
						}else{
							$srcset .= $cropped_image . ' ' . $cropped_size . 'w';
						}
						$smallest_image = array($cropped_image, $cropped_size, $new_height);
					}
				}
			}			
	
			if( !empty($srcset) ){
				$ret  = ' src="' . esc_url($image[0]) . '" width="' . esc_attr($image[1]) . '" height="' . esc_attr($image[2]) . '" ';
				// $ret  = ' src="' . esc_url($smallest_image[0]) . '" width="' . esc_attr($image[1]) . '" height="' . esc_attr($image[2]) . '" ';
				if( false ){
					$ret .= ' srcset="' . esc_attr($srcset) . ', ' . esc_attr(rocket_cdn_file($image[0])) . ' ' . esc_attr($image[1]) . 'w" ';
				}else{
					$ret .= ' srcset="' . esc_attr($srcset) . ', ' . esc_attr($image[0]) . ' ' . esc_attr($image[1]) . 'w" ';
				}
				// get screen size for query
				global $content_width, $gdlr_core_container, $gdlr_core_container_multiplier, $gdlr_core_item_multiplier;
				if( empty($gdlr_core_container) ){ gdlr_core_set_container(); }
				$column_size = intval(100 * $gdlr_core_container_multiplier * $gdlr_core_item_multiplier);
				$content_size = intval($gdlr_core_container * $gdlr_core_container_multiplier * $gdlr_core_item_multiplier);
				
				$sizes = '(max-width: 767px) 100vw';
				if( $gdlr_core_container >= 2560 ){
					$sizes .= ', ' . $column_size . 'vw';
				}else{
					$sizes .= ', (max-width: ' . $gdlr_core_container . 'px) ' . $column_size . 'vw';
					$sizes .= ', ' . $content_size . 'px';
				}
				
				$ret .= ' sizes="' . esc_attr($sizes) . '" ';
				return $ret;
			}

			return '';
		}
	}

	if( !function_exists('gdlr_core_get_cropped_image') ){
		function gdlr_core_get_cropped_image( $attachment_id = 0, $width = '', $height = '', $html = true ){
			if( empty($attachment_id) ){
				return;
			}

			$original_path = get_attached_file($attachment_id);
			if( empty($original_path) ){
				return;
			}

			$orig_info = pathinfo($original_path);
			$dir = $orig_info['dirname'];
			$ext = $orig_info['extension'];	

			$suffix = "{$width}x{$height}";
			$name = wp_basename($original_path, ".{$ext}");
			$destfilename = "{$dir}/{$name}-{$suffix}.{$ext}";

			$attachment = wp_get_attachment_image_src($attachment_id, 'full');
			$destfileurl = str_replace($name, $name . '-' . $suffix, $attachment[0]);

			if( !file_exists($destfilename) ){

				// get attachment for resize && check if it's resizable
				$attachment_thumbnail = wp_get_attachment_image_src($attachment_id, 'thumbnail');
				if( $attachment[1] == $attachment_thumbnail[1] && $attachment[2] == $attachment_thumbnail[2] ){
					return;
				}
			
				// crop an image
				$cropped_image = wp_get_image_editor($original_path);
				if( !is_wp_error($cropped_image) ) {
					$cropped_image->resize($width, $height, true);
					$cropped_image->save($destfilename);

					if( !$html ){
						return $destfileurl;
					}else{
						$alt_text = get_post_meta($attachment_id , '_wp_attachment_image_alt', true);
						return '<img src="' . esc_url($destfileurl) . '" width="' . esc_attr($width) . '" height="' . esc_attr($height) . '" alt="' . (empty($alt_text)? '': $alt_text) . '" >';
					}
				}
			}else{
				if( !$html ){
						return $destfileurl;
				}else{
					$alt_text = get_post_meta($attachment_id , '_wp_attachment_image_alt', true);
					return '<img src="' . esc_url($destfileurl) . '" width="' . esc_attr($width) . '" height="' . esc_attr($height) . '" alt="' . (empty($alt_text)? '': $alt_text) . '" >';
				}
			}

		} // gdlr_core_get_cropped_image
	} // function_exists


	if( !function_exists('gdlr_core_to_shortcode') ){
		function gdlr_core_to_shortcode( $name, $atts, $content = null ){
			$ret = '[' . $name . ' ';
			foreach( $atts as $att_key => $att_val ){
				$ret .= $att_key . '="' . esc_attr($att_val) . '" ';
			}
			$ret .= ']';

			if( $content !== null ){
				$ret .= esc_html($content) . '[/' . $name . ']';
			}

			return $ret;
		}
	}

	add_action('wp_head', 'gdlr_core_head_inline_style');
	if( !function_exists('gdlr_core_head_inline_style') ){
		function gdlr_core_head_inline_style(){
			global $gdlr_core_inline_style_cache;
			$gdlr_core_inline_style_cache = get_post_meta(get_the_ID(), 'gdlr-core-pb-inline-css', true);

			if( !empty($gdlr_core_inline_style_cache) ){
				echo '<style>'. $gdlr_core_inline_style_cache . '</style>';
			}
		}
	}
	add_action('wp_footer', 'gdlr_core_inline_style');
	if( !function_exists('gdlr_core_inline_style') ){
		function gdlr_core_inline_style(){
			global $gdlr_core_inline_style, $gdlr_core_inline_style_cache;

			if( $gdlr_core_inline_style_cache != $gdlr_core_inline_style ){
				update_post_meta(get_the_ID(), 'gdlr-core-pb-inline-css', $gdlr_core_inline_style);

				if( !empty($gdlr_core_inline_style) ){
					echo '<style>'. $gdlr_core_inline_style . '</style>';
				}
			}
		}
	}
	if( !function_exists('gdlr_core_add_inline_style') ){
		function gdlr_core_add_inline_style( $css ){
			global $gdlr_core_inline_style;

			$gdlr_core_inline_style  = empty($gdlr_core_inline_style)? '': $gdlr_core_inline_style;
			$gdlr_core_inline_style .= $css;
		}
	}

	if( !function_exists('gdlr_core_option_to_css') ){
		function gdlr_core_option_to_css($option_slug, $option, $option_val){

			$ret = '';

			if( empty($option['data-type']) ){
				$option['data-type'] = 'color';
			}else if( $option['data-type'] == 'rgba' ){

				// replace the rgba first
				$value = gdlr_core_format_datatype($option_val[$option_slug], 'rgba');
				$option['selector'] = str_replace('#gdlra#', $value, $option['selector']);
				
				$option['data-type'] = 'color';

			}
			$value = gdlr_core_format_datatype($option_val[$option_slug], $option['data-type']);

			// for secondary selector
			if( !empty($option['selector-extra']) ){ 

				while( $start_extra = strpos($option['selector'], '<') ){
					$end_extra = strpos($option['selector'], '>', $start_extra);
					$end_alpha = strpos($option['selector'], '>a', $start_extra);
					$end_text = strpos($option['selector'], '>t', $start_extra);

					if( $start_extra !== false && $end_extra !== false ){
						$custom_slug = substr($option['selector'], ($start_extra + 1), ($end_extra - $start_extra - 1));
						$option_val[$custom_slug] = empty($option_val[$custom_slug])? '': $option_val[$custom_slug];
						
						if( $end_alpha !== false ){
							$custom_value = gdlr_core_format_datatype($option_val[$custom_slug], 'rgba');
							$option['selector'] = str_replace('<' . $custom_slug . '>a', $custom_value, $option['selector']);
						}else if( $end_text !== false ){
							$custom_value = gdlr_core_format_datatype($option_val[$custom_slug], 'text');
							$option['selector'] = str_replace('<' . $custom_slug . '>t', $custom_value, $option['selector']);
						}else{
							$custom_value = gdlr_core_format_datatype($option_val[$custom_slug], $option['data-type']);
							$option['selector'] = str_replace('<' . $custom_slug . '>', $custom_value, $option['selector']);
						}
					}
				}
			}

			if( empty($value) && $option['data-type'] == 'image' ){
				
			}else{
				$ret .= str_replace('#gdlr#', $value, $option['selector']) . " \n";
			}

			return $ret;

		}
	}