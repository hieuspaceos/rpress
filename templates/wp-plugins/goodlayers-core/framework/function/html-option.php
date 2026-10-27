<?php
	/*	
	*	Goodlayers Html Option File
	*	---------------------------------------------------------------------
	*	This file create the class that help you create the input form element
	*	---------------------------------------------------------------------
	*/	
	
	if( !class_exists('gdlr_core_html_option') ){
		
		class gdlr_core_html_option{

			// call this function on wp_enqueue_script hook
			static function include_script($elements = array()){
				
				$elements = wp_parse_args($elements, array(
					'style' => 'html-option',
				));				

				gdlr_core_include_utility_script();

				wp_enqueue_media();
				wp_enqueue_style('wp-color-picker');
				wp_enqueue_style('gdlr-core-html-option', GDLR_CORE_URL . '/framework/css/' . $elements['style'] . '.css');
				
				// enqueue the script
				wp_enqueue_script('gdlr-core-html-option', GDLR_CORE_URL . '/framework/js/html-option.js', array(
					'jquery', 'jquery-effects-core', 'wp-color-picker', 'jquery-ui-slider', 'jquery-ui-datepicker'
				), '1.4.6', true);	
				
				// localize the script
				$html_option_val =  array();
				$html_option_val['icon_list'] = self::icons_js();
				$html_option_val['text'] = array(
					'ajaxurl' => GDLR_CORE_AJAX_URL,
					'error_head' => esc_html__('An error occurs', 'goodlayers-core'),
					'error_message' => esc_html__('Please refresh the page to try again. If the problem still persists, please contact administrator for this.', 'goodlayers-core'),
					'nonce' => wp_create_nonce('gdlr_core_html_option'),
					'upload_media' => esc_html__('Select or Upload Media', 'goodlayers-core'),
					'choose_media' => esc_html__('Use this media', 'goodlayers-core'),
				);
				$html_option_val['tabs'] = array(
					'title_text' => esc_html__('Item\'s Title', 'goodlayers-core'),
					'tab_checkbox_on' => esc_html__('On', 'goodlayers-core'),
					'tab_checkbox_off' => esc_html__('Off', 'goodlayers-core')
				);
				$html_option_val['skin'] = array(
					'input' => esc_html__('Skin Name', 'goodlayers-core'),
					'empty_input' => esc_html__('Please fill the name in skin name box to create new skin.', 'goodlayers-core'),
					'duplicate_input' => esc_html__('This skin name has already been assigned, please try filling another name.', 'goodlayers-core'),
					'description' => esc_html__('* Please fill english character for skin name with no special characters. The skin you\'re created can be used in Color/Background Wrapper Section', 'goodlayers-core')
				);
				$html_option_val['fontupload'] = array(
					'none' => esc_html__('You don\'t have any font uploaded', 'goodlayers-core'),
					'font_name' => esc_html__('Font Name', 'goodlayers-core'),
					'font_name_p' => esc_html__('Fill in font name in English', 'goodlayers-core'),
					'eot' => esc_html__('EOT Font', 'goodlayers-core'),
					'ttf' => esc_html__('TTF Font', 'goodlayers-core'),
					'woff' => esc_html__('Woff Font', 'goodlayers-core'),
					'font_weight' => esc_html__('Font Weight', 'goodlayers-core'),
					'font_style' => esc_html__('Font Style', 'goodlayers-core'),
					'button' => esc_html__('Upload', 'goodlayers-core'),
				);
				$html_option_val['thumbnail_sizing'] = array(
					'name' => esc_html__('Thumbnail Name', 'goodlayers-core'),
					'width' => esc_html__('Width (px)', 'goodlayers-core'),
					'height' => esc_html__('Height (px)', 'goodlayers-core'),
					'add' => esc_html__('Add Thumbnail', 'goodlayers-core'),
					'empty_input' => esc_html__('Please fill all required fields', 'goodlayers-core'),
					'description' => esc_html__('*After creating new thumbnail, you have to regenerate the thumbnail for old images.', 'goodlayers-core') . ' ' .
						esc_html__('We recommend the \'ONet Regenerate thumbnails\' plugin for this process.', 'goodlayers-core')
						
				);
				wp_localize_script('gdlr-core-html-option', 'html_option_val', $html_option_val);

				// for tmce initialization
				// $html_option_val['tmce'] = self::tmce_init();
				add_action('admin_head', 'gdlr_core_html_option::late_include_script', 999);
			}
			static function late_include_script(){
				$gdlr_core_tmce = self::tmce_init();
?>
<script>
	var gdlr_core_tmce = <?php echo json_encode($gdlr_core_tmce); ?>;
</script>
<?php
			}
			
			// use to obtain input elements based on the settings variable
			static function get_element($settings){
				
				if( empty($settings['type']) || $settings['type'] == 'customizer-description' ) return;
				
				$wrapper_class  = empty($settings['wrapper-class'])? '': $settings['wrapper-class'];
				$wrapper_class .= ' gdlr-core-html-option-' . trim($settings['type']);
				$condition = empty($settings['condition'])? '': 'data-condition="' . esc_attr(json_encode($settings['condition'])) . '"';
				
				$ret  = '<div class="gdlr-core-html-option-item ' . esc_attr($wrapper_class) . '-item" ' . $condition . ' >';
				
				if( !empty($settings['title']) ){
					$ret .= '<div class="gdlr-core-html-option-item-title" >' . gdlr_core_escape_content($settings['title']) . '</div>';
				}

				if( !empty($settings['description-position']) && $settings['description-position'] == 'top' && !empty($settings['description']) ){
					$ret .= '<div class="gdlr-core-html-option-item-description" >' . gdlr_core_escape_content($settings['description']) . '</div>';
				}
				
				$ret .= '<div class="gdlr-core-html-option-item-input">';
				switch($settings['type']){
					case 'text': 
						$ret .= self::text($settings);
						break;
					case 'datepicker': 
						$ret .= self::datepicker($settings);
						break;
					case 'textarea': 
						$ret .= self::textarea($settings);
						break;
					case 'combobox':
						$ret .= self::combobox($settings);
						break;
					case 'multi-combobox':
						$ret .= self::multi_combobox($settings);
						break;
					case 'checkbox': 
						$ret .= self::checkbox($settings);
						break;
					case 'radioimage': 
					case 'radioimage-frame': 
						$ret .= self::radioimage($settings);
						break;
					case 'upload': 
						$ret .= self::upload($settings);
						break;
					case 'colorpicker': 
						$ret .= self::colorpicker($settings);
						break;
					case 'font': 
						$ret .= self::font($settings);
						break;
					case 'fontslider': 
						$ret .= self::fontslider($settings);
						break;
					case 'tinymce': 
						$ret .= self::tinymce($settings);
						break;
					case 'icons': 
						$ret .= self::icons($settings);
						break;
					case 'custom': 
						$ret .= self::custom($settings);
						break;
					case 'import': 
						$ret .= self::import($settings);
						break;
					case 'export': 
						$ret .= self::export($settings);
						break;
					default: break;
				}
				$ret .= '</div>';
				
				if( empty($settings['description-position']) && !empty($settings['description']) ){
					$ret .= '<div class="gdlr-core-html-option-item-description" >' . gdlr_core_escape_content($settings['description']) . '</div>';
				}
				
				if( !empty($settings['options']) && $settings['options'] == 'skin' ){
					$ret .= '<div class="gdlr-core-html-option-skin-edit" >' . esc_html__('Create Skin', 'goodlayers-core') . '<i class="fa fa-plus-circle" ></i></div>';
				}

				$ret .= '<div class="clear"></div>';
				$ret .= '</div>'; // gdlr-core-html-option-item
				
				return $ret;
			}
			
			//////////////////////////
			// element started here
			//////////////////////////			
			
			// input text
			static function text($settings){
				$value = '';
				
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( isset($settings['default']) ){
					$value = $settings['default'];
				}

				$ret  = '<input type="text" class="gdlr-core-html-option-text" data-type="text" data-slug="' . esc_attr($settings['slug']) . '" value="' . esc_attr($value) . '" ';
				$ret .= (!empty($settings['with-name']))? ' name="' . esc_attr($settings['slug']) . '" ': '';
				$ret .= empty($settings['data-input-type'])? '': ' data-input-type="' . esc_attr($settings['data-input-type']) . '"';
				$ret .= ' />';
	
				return $ret;
			}	

			// input datepicker
			static function datepicker($settings){
				$value = '';
				
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( isset($settings['default']) ){
					$value = $settings['default'];
				}

				$ret  = '<input type="text" class="gdlr-core-html-option-text gdlr-core-html-option-datepicker" data-type="text" data-slug="' . esc_attr($settings['slug']) . '" value="' . esc_attr($value) . '" />';
				$ret .= '<i class="gdlr-core-html-option-datepicker-icon fa fa-calendar" ></i>';
				return $ret;
			}			
			
			// textarea
			static function textarea($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}

				$ret = '<textarea class="gdlr-core-html-option-textarea" data-type="textarea" data-slug="' . esc_attr($settings['slug']) . '" >' . esc_textarea($value) . '</textarea>';
	
				return $ret;
			}
			
			// combobox
			static function combobox($settings){
				$value = '';
				$extra_html = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}
				
				$ret  = '<div class="gdlr-core-custom-combobox" >';
				$ret .= '<select class="gdlr-core-html-option-combobox" data-type="combobox" ';
				$ret .= (!empty($settings['with-name']))? ' name="' . esc_attr($settings['slug']) . '" ': '';
				$ret .= 'data-slug="' . esc_attr($settings['slug']) . '" >';
				if( !empty($settings['options']) ){
					if( $settings['options'] == 'sidebar' ){
						$settings['options'] =  array('' => esc_html__('None', 'goodlayers-core')) + gdlr_core_sidebar_generator::get_sidebars();
					}else if( $settings['options'] == 'thumbnail-size' ){
						$settings['options'] = gdlr_core_get_thumbnail_list();
					}else if( $settings['options'] == 'skin' ){
						$settings['options'] = gdlr_core_skin_settings::get_skins();
					}else if( $settings['options'] == 'post_type' ){
						$settings['options'] = gdlr_core_get_post_list($settings['options-data']);
						if( !empty($settings['allow-none']) ){
							$settings['options'] = array('' => esc_html__('None', 'goodlayers-core')) + $settings['options'];
						}
					}else if( $settings['options'] == 'tax_id' ){
						$settings['options'] = gdlr_core_get_term_list_id($settings['options-data']);
					}
					
					if( !empty($settings['with-default']) ){
						$settings['options'] = array(
							'default' => esc_html__('Default', 'goodlayers-core')
						) + $settings['options'];
					}

					foreach($settings['options'] as $option_key => $option_value ){
						$ret .= '<option value="' . esc_attr($option_key) . '" ' . selected($value, $option_key, false) . ' >' . gdlr_core_escape_content($option_value) . '</option>';
					}
				}
				$ret .= '</select>';
				$ret .= '</div>';
				
				return $ret;
			}
			
			// multi_combobox
			static function multi_combobox($settings){
				$value = array();
				if( isset($settings['value']) ){
					if( is_array($settings['value']) ){
						$value = $settings['value'];
					}else if( !empty($settings['value']) ){
						$value = explode(',', $settings['value']);
					}
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}
				
				$ret  = '<select class="gdlr-core-html-option-multi-combobox" ';
				$ret .= (!empty($settings['with-name']))? ' name="' . esc_attr($settings['slug']) . '[]" ': '';
				$ret .= 'data-type="multi-combobox" data-slug="' . esc_attr($settings['slug']) . '" multiple >';
				if( !empty($settings['options']) ){
					foreach($settings['options'] as $option_key => $option_value ){
						$ret .= '<option value="' . esc_attr($option_key) . '" ' . ($value == 'all' || in_array($option_key, $value)? 'selected': '') . ' >' . gdlr_core_escape_content($option_value) . '</option>';
					}
				}
				$ret .= '</select>';
				
				return $ret;
			}			
			
			// checkbox
			static function checkbox($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}else{
					$value = 'enable';
				}
				
				$ret  = '<label>';
				$ret .= '<input type="checkbox" class="gdlr-core-html-option-checkbox" data-type="checkbox" data-slug="' . esc_attr($settings['slug']) . '" ' . checked($value, 'enable', false) . ' />';
				$ret .= '<div class="gdlr-core-html-option-checkbox-appearance gdlr-core-noselect">';
				$ret .= '<span class="gdlr-core-checkbox-button gdlr-core-on">' . esc_html__('On', 'goodlayers-core') . '</span>';
				$ret .= '<span class="gdlr-core-checkbox-separator"></span>';
				$ret .= '<span class="gdlr-core-checkbox-button gdlr-core-off">' . esc_html__('Off', 'goodlayers-core') . '</span>';
				$ret .= '</div>';
				$ret .= '</label>';
				
				return $ret;
			}		
			
			// radioimage
			static function radioimage($settings){

				if( $settings['options'] == 'text-align' ){
					$settings['options'] = array(
						'left' => GDLR_CORE_URL . '/include/images/text-align/left.png',
						'center' => GDLR_CORE_URL . '/include/images/text-align/center.png',
						'right' => GDLR_CORE_URL . '/include/images/text-align/right.png'
					);
					$settings['max-width'] = '61px';
					$settings['type'] = 'radioimage-frame';

					if( !empty($settings['with-default']) ){
						$settings['options'] = array_merge(array(
							'default' => GDLR_CORE_URL . '/include/images/text-align/default.jpg',
						), $settings['options']);
					}
				}else if( $settings['options'] == 'sidebar' ){
					$settings['options'] = array(
						'none' => GDLR_CORE_URL . '/include/images/sidebar/none.jpg',
						'left' => GDLR_CORE_URL . '/include/images/sidebar/left.jpg',
						'right' => GDLR_CORE_URL . '/include/images/sidebar/right.jpg',
						'both' => GDLR_CORE_URL . '/include/images/sidebar/both.jpg',
					);

					if( !empty($settings['with-default']) ){
						$settings['options'] = array_merge(array(
							'default' => GDLR_CORE_URL . '/include/images/sidebar/default.jpg',
						), $settings['options']);
					}
					if( !empty($settings['without-none']) ){
						unset($settings['options']['none']);
					}
				}else if( $settings['options'] == 'pattern' ){
					$settings['options'] = array(
						'pattern-1' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-1.png',
						'pattern-2' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-2.png',
						'pattern-3' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-3.png',
						'pattern-4' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-4.png',
						'pattern-5' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-5.png',
						'pattern-6' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-6.png',
						'pattern-7' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-7.png',
						'pattern-8' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-8.png',
						'pattern-9' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-9.png',
						'pattern-10' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-10.png',
						'pattern-11' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-11.png',
						'pattern-12' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-12.png',
						'pattern-13' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-13.png',
						'pattern-14' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-14.png',
						'pattern-15' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-15.png',
						'pattern-16' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-16.png',
						'pattern-17' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-17.png',
						'pattern-18' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-18.png',
						'pattern-19' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-19.png',
						'pattern-20' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-20.png',
						'pattern-21' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-21.png',
						'pattern-22' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-22.png',
						'pattern-23' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-23.png',
						'pattern-24' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-24.png',
						'pattern-25' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-25.png',
						'pattern-26' => GDLR_CORE_URL . '/include/images/pattern/thumbnail/pattern-26.png'
					);
				}else if( $settings['options'] == 'hover-icon-link' ){
					$settings['options'] = array(
						'arrow_right-up' => GDLR_CORE_URL . '/include/images/hover-icon/link/arrow_right-up.jpg',
						'fa fa-external-link' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-external-link.jpg',
						'fa fa-external-link-square' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-external-link-square.jpg',
						'fa fa-link' => GDLR_CORE_URL . '/include/images/hover-icon/link/fa-link.jpg',
						'icon_link' => GDLR_CORE_URL . '/include/images/hover-icon/link/icon_link.jpg',
						'icon_link_alt' => GDLR_CORE_URL . '/include/images/hover-icon/link/icon_link_alt.jpg'
					);
					$settings['max-width'] = '30px';
					$settings['type'] = 'radioimage-frame';
				}else if( $settings['options'] == 'hover-icon-image' ){
					$settings['options'] = array(
						'arrow_expand' => GDLR_CORE_URL . '/include/images/hover-icon/image/arrow_expand.jpg',
						'fa fa-expand' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-expand.jpg',
						'fa fa-picture-o' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-picture-o.jpg',
						'fa fa-plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-plus.jpg',
						'fa fa-plus-circle' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-plus-circle.jpg',
						'fa fa-search' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-search.jpg',
						'fa fa-search-plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/fa-search-plus.jpg',
						'icon_plus' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_plus.jpg',
						'icon_plus_alt2' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_plus_alt2.jpg',
						'icon_search' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_search.jpg',
						'icon_zoom-in_alt' => GDLR_CORE_URL . '/include/images/hover-icon/image/icon_zoom-in_alt.jpg',
					);
					$settings['max-width'] = '30px';
					$settings['type'] = 'radioimage-frame';
				}else if( $settings['options'] == 'hover-icon-video' ){
					$settings['options'] = array(
						'fa fa-file-video-o' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-file-video-o.jpg',
						'fa fa-film' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-film.jpg',
						'fa fa-play' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play.jpg',
						'fa fa-play-circle' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play-circle.jpg',
						'fa fa-play-circle-o' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-play-circle-o.jpg',
						'fa fa-video-camera' => GDLR_CORE_URL . '/include/images/hover-icon/video/fa-video-camera.jpg',
						'icon_film' => GDLR_CORE_URL . '/include/images/hover-icon/video/icon_film.jpg',
					);
					$settings['max-width'] = '30px';
					$settings['type'] = 'radioimage-frame';
				}

				$value = '';
				if( !empty($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}else{
					reset($settings['options']);
					$value = key($settings['options']);
				}
				
				$max_width = empty($settings['max-width'])? '': gdlr_core_format_datatype($settings['max-width'], 'pixel');
				$ret = '';
				foreach( $settings['options'] as $option_key => $option_url ){
					$ret .= '<label ' . gdlr_core_esc_style(array('max-width'=> $max_width)) . ' >';
					$ret .= '<input class="gdlr-core-html-option-radioimage" type="radio" name="' . esc_attr($settings['slug']) . '" data-type="radioimage" data-slug="' . esc_attr($settings['slug']) . '" value="' . esc_attr($option_key) . '" ' . checked($value, $option_key, false) . '/>';
					if( $settings['type'] == 'radioimage-frame' ){
						$ret .= '<div class="gdlr-core-radioimage-frame" ></div>';
					}else{
						$ret .= '<div class="gdlr-core-radioimage-checked" ></div>';
					}
					$ret .= '<img src="' . esc_url($option_url) . '" alt="' . esc_attr($option_key) . '" />';
					$ret .= '</label>';
				}
				
				return $ret;
			}
			
			// upload
			static function upload($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}
				
				$ret  = '<div class="gdlr-core-html-option-upload-appearance ' . (empty($value)? '': 'gdlr-core-active') . '" >';
				$ret .= '<input type="hidden" class="gdlr-core-html-option-upload" data-type="upload" data-slug="' . esc_attr($settings['slug']) . '" value="' . esc_attr($value) . '" ';
				$ret .= (!empty($settings['with-name']))? ' name="' . esc_attr($settings['slug']) . '" ': '';
				$ret .= ' />';
				
				$ret .= '<div class="gdlr-core-upload-image-container" style="' . (empty($value)? '': 'background-image: url(\'' . esc_url(wp_get_attachment_url($value)) . '\');') . '" ></div>';
				
				$ret .= '<div class="gdlr-core-upload-image-overlay" >';
				$ret .= '<div class="gdlr-core-upload-image-button-hover">';
				$ret .= '<span class="gdlr-core-upload-image-button gdlr-core-upload-image-add"><i class="icon_plus" ></i></span>';
				$ret .= '<span class="gdlr-core-upload-image-button gdlr-core-upload-image-remove"><i class="icon_minus-06" ></i></span>';
				$ret .= '</div>'; // gdlr-core-upload-image-hover
				$ret .= '</div>'; // gdlr-core-upload-image-overlay
				$ret .= '</div>'; // gdlr-core-html-option-upload-appearance
				
				return $ret;
			}
			
			// colorpicker
			static function colorpicker($settings){
				$value = ''; $default = '';
				if( !empty($settings['default']) ){
					$default = $settings['default'];
				}
				
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($default) ){
					$value = $default;
				}
				
				$ret = '<input type="text" class="gdlr-core-html-option-colorpicker" data-type="colorpicker" data-slug="' . esc_attr($settings['slug']) . '" value="' . esc_attr($value) . '" data-default-color="' . esc_attr($default) . '" />';
	
				return $ret;
			}
			
			// font
			static function font($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else{
					$value = empty($settings['default'])? 'Helvetica, sans-serif': $settings['default'];
				}
				
				// init the font if not exists
				global $gdlr_core_font_loader;
				if( empty($gdlr_core_font_loader) ){
					$gdlr_core_font_loader = new gdlr_core_font_loader();
				}

				$base_url = gdlr_core_get_font_display_page();
				$display_url = add_query_arg(array('font-family'=>$value, 'font-type'=>'none'), $base_url);
				
				$ret  = '<iframe class="gdlr-core-html-option-font-display" src="' . esc_url($display_url) . '" data-base-url="' . esc_attr($base_url) . '" ></iframe>';
				$ret .= '<div class="gdlr-core-custom-combobox" >';
				$ret .= '<select class="gdlr-core-html-option-font" data-type="font" data-slug="' . esc_attr($settings['slug']) . '" >';
				
				$ret .= $gdlr_core_font_loader->get_option_list($value);
				
				$ret .= '</select>';
				$ret .= '</div>';				
				
				return $ret;
			}
			
			// fontslider
			static function fontslider($settings){
				$value = '';
				if( !empty($settings['value']) || (isset($settings['value']) && $settings['value'] === '0') ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}else{
					$value = 0;
				}

				if( !empty($settings['data-type']) && $settings['data-type'] == 'opacity' ){
					$settings['data-min'] = 0;
					$settings['data-max'] = 100;
					$settings['data-suffix'] = 'none';
				}
				
				$ret  = '<input type="text" class="gdlr-core-html-option-fontslider" data-type="text" value="' . esc_attr($value) . '" ';
				$ret .= 'data-slug="' . esc_attr($settings['slug']) . '" ';
				$ret .= isset($settings['data-min'])? 'data-min-value="' . esc_attr($settings['data-min']) . '" ': '';
				$ret .= isset($settings['data-max'])? 'data-max-value="' . esc_attr($settings['data-max']) . '" ': '';
				$ret .= isset($settings['data-suffix'])? ' data-suffix="' . esc_attr($settings['data-suffix']) . '" ': '';
				$ret .= ' />';
				
				return $ret;
			}
			
			// icons
			static function icons_js(){
				
				$use_font_icons = apply_filters('gdlr_core_use_font_icons', array('font-awesome', 'elegant-font'));
				
				$ret  = '<div class="gdlr-core-custom-combobox gdlr-core-html-option-icons-type-combobox" >';
				$ret .= '<select class="gdlr-core-html-option-combobox gdlr-core-html-option-icons-type-select" >';
				foreach($use_font_icons as $use_font_icon){
					$ret .= '<option value="' . esc_attr($use_font_icon) .  '" >' . gdlr_core_get_icon_font_title($use_font_icon) . '</option>';
				}
				$ret .= '</select>';
				$ret .= '</div>';
				
				$ret .= '<input type="text" class="gdlr-core-html-option-text gdlr-core-html-option-icons-search" placeholder="' . esc_html__('Search Icons', 'goodlayers-core') . '" />';
				$ret .= '<div class="gdlr-core-html-option-icons-type-wrapper" >';
				
				$first = true;
				foreach($use_font_icons as $use_font_icon){
					$icon_list = gdlr_core_get_icon_font_list($use_font_icon);

					$ret .= '<div class="gdlr-core-html-option-icons-type ' . ($first? 'gdlr-core-active': '') . '" data-icon-type="' . esc_attr($use_font_icon) . '" >';
					if( !empty($icon_list) && is_array($icon_list) ){
						foreach( $icon_list as $icon ){
							$ret .= '<i class="' . esc_attr($icon) . '" ></i>';
						}
					}
					$ret .= '</div>';

					$first = false;
				}
				
				$ret .= '</div>'; // gdlr-core-html-option-icon-type-wrapper

				return $ret;

			}
			static function icons($settings){
				$ret = '';
				$value = '';
				
				if( !empty($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}
				
				$use_font_icons = apply_filters('gdlr_core_use_font_icons', array('font-awesome', 'elegant-font'));

				$settings['allow-none'] = empty($settings['allow-none'])? false: $settings['allow-none'];
				if( $value == 'none' ){
					$font_type = 'none';
				}else{
					$font_type = gdlr_core_get_icon_font_type($value, $settings['allow-none']);
					if( !in_array($font_type, $use_font_icons) ){
						$font_type = $use_font_icons[0];
					}
				}
				
				$ret .= '<div class="gdlr-core-custom-combobox gdlr-core-html-option-icons-type-combobox" >';
				$ret .= '<select class="gdlr-core-html-option-combobox gdlr-core-html-option-icons-type-select" >';
				if( !empty($settings['allow-none']) ){
					$ret .= '<option value="none" ' . ($font_type == 'none'? 'selected': '') . ' >' . esc_html__('None', 'goodlayers-core') . '</option>';
				}

				foreach($use_font_icons as $use_font_icon){
					$ret .= '<option value="' . esc_attr($use_font_icon) .  '" ' . ($font_type == $use_font_icon? 'selected': '') . ' >' . gdlr_core_get_icon_font_title($use_font_icon) . '</option>';
				}
				$ret .= '</select>';
				$ret .= '</div>';
				
				$ret .= '<input type="text" class="gdlr-core-html-option-text gdlr-core-html-option-icons-search" placeholder="' . esc_html__('Search Icons', 'goodlayers-core') . '" ';
				if( $font_type == 'none' ){
					$ret .= gdlr_core_esc_style(array('display' => 'none'));
				}
				$ret .= ' />';

				$ret .= '<div class="gdlr-core-html-option-icons-type-wrapper" ';
				if( $font_type == 'none' ){
					$ret .= gdlr_core_esc_style(array('display' => 'none'));
				}
				$ret .= ' >';
				
				foreach($use_font_icons as $use_font_icon){
					$icon_list = gdlr_core_get_icon_font_list($use_font_icon);

					$ret .= '<div class="gdlr-core-html-option-icons-type';
					$ret .= ($font_type == $use_font_icon? ' gdlr-core-active': '') . '" ';
					$ret .= ' data-icon-type="' . esc_attr($use_font_icon) . '" >';
					foreach( $icon_list as $icon ){
						$ret .= '<i class="' . esc_attr($icon) . ($value == $icon? ' gdlr-core-active': '') . '" ></i>';
					}
					$ret .= '</div>';
				}
				
				$ret .= '</div>'; // gdlr-core-html-option-icon-type-wrapper
				
				$ret .= '<input type="hidden" value="' . esc_attr($value) . '" data-type="text" ';
				$ret .= 'data-slug="' . esc_attr($settings['slug']) . '" ';
				$ret .= empty($settings['with-name'])? '': 'name="' . esc_attr($settings['slug']) . '" ';
				$ret .= '/>';

				return $ret;
			}
			
			// custom
			static function custom($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}

				$ret  = '<div class="gdlr-core-html-option-custom" data-type="custom" data-item-type="' . esc_attr($settings['item-type']) . '" data-slug="' . esc_attr($settings['slug']) . '" ';
				$ret .= empty($settings['data-input-type'])? '': ' data-input-type="' . esc_attr($settings['data-input-type']) . '" ';
				$ret .= empty($settings['extra'])? '': ' data-extra="' . esc_attr(json_encode($settings['extra'])) . '" ';
				$ret .= '>';
				if( !empty($settings['options']) ){
					$ret .= '<span class="gdlr-core-html-option-custom-options" data-value="' . esc_attr(json_encode($settings['options'])) . '" ></span>';
				}
				if( !empty($value) ){
					$ret .= '<span class="gdlr-core-html-option-custom-value" data-value="' . esc_attr(json_encode($value)) . '" ></span>';
				}
				$ret .= '</div>';
	
				return $ret;
			}	

			// import
			static function import($settings){

				$ret  = '<div class="gdlr-core-html-option-import" data-action="' . esc_attr($settings['action']) . '" >';
				$ret .= '<form method="post" enctype="multipart/form-data" >';
				$ret .= '<input class="gdlr-core-html-option-import-file" type="file" name="gdlr-core-import" >';
				$ret .= '<div class="gdlr-core-html-option-import-button" >' . esc_html__('Import', 'goodlayers-core') . '</div>';
				$ret .= '</form>';
				$ret .= '</div>';
	
				return $ret;
			}

			// export
			static function export($settings){

				$ret  = '<div class="gdlr-core-html-option-export" data-action="' . esc_attr($settings['action']) . '" >';
				if( !empty($settings['options']) ){
					$ret .= '<div class="gdlr-core-custom-combobox" >';
					$ret .= '<select class="gdlr-core-html-option-export-option gdlr-core-html-option-combobox" data-type="combobox" >';
					if( !empty($settings['options']) ){
						foreach($settings['options'] as $option_key => $option_value ){
							$ret .= '<option value="' . esc_attr($option_key) . '" >' . gdlr_core_escape_content($option_value) . '</option>';
						}
					}
					$ret .= '</select>';
					$ret .= '</div>';
				}
				$ret .= '<div class="gdlr-core-html-option-export-button" >' . esc_html__('Export', 'goodlayers-core') . '</div>';
				$ret .= '</div>';
	
				return $ret;
			}
			
			//////////////////////////////////////////////
			// tinymce
			// ref: wp-includes/class-wp-editor.php
			//////////////////////////////////////////////
			static function tinymce($settings){
				$value = '';
				if( isset($settings['value']) ){
					$value = $settings['value'];
				}else if( !empty($settings['default']) ){
					$value = $settings['default'];
				}

				$ret  = '<div class="gdlr-core-html-option-tinymce" data-type="tinymce" data-slug="' . esc_attr($settings['slug']) . '" >';
				$ret .= gdlr_core_escape_content($value);
				$ret .= '</div>';
	
				return $ret;
			}		
			static function tmce_init(){
				
				if( !class_exists('_WP_Editors', false) ){
					require( ABSPATH . WPINC . '/class-wp-editor.php' );
				}
				
				$editor_id = 'gdlr_core_tmce';
				$set = _WP_Editors::parse_settings($editor_id, array());
				$set['editor_class'] .= ' wp-editor-area';
				$set['media_buttons'] = current_user_can('upload_files')? true: false;
				$set['default_editor'] = (wp_default_editor() == 'html')? 'html': 'tinymce';
				$set['switch_button'] = user_can_richedit();
				$set['switch_class'] = ($set['default_editor'] == 'tinymce' &&  $set['switch_button'])? 'tmce-active': 'html-active';
				
				$pb_tmce  = '<div id="wp-' . esc_attr($editor_id) . '-wrap" class="wp-core-ui wp-editor-wrap ' . esc_attr($set['switch_class']) . '">';
				$pb_tmce .= empty($set['editor_css'])? '': $set['editor_css']; 
				if( !empty($set['switch_button']) || !empty($set['media_buttons']) ){
					$pb_tmce .= '<div id="wp-' . esc_attr($editor_id) . '-editor-tools" class="wp-editor-tools hide-if-no-js">';
					if( !empty($set['media_buttons']) ){
						$pb_tmce .= '<div id="wp-' . esc_attr($editor_id) . '-media-buttons" class="wp-media-buttons">';
						
						if( !function_exists('media_buttons') ){
							include(ABSPATH . 'wp-admin/includes/media.php');
						}
						
						ob_start();
						do_action('media_buttons', $editor_id);
						$pb_tmce .= ob_get_contents();
						ob_end_clean();
						
						$pb_tmce .= '</div>'; // wp-media-buttons
					}
					$pb_tmce .= '<div class="wp-editor-tabs">';
					if( $set['switch_button'] ){
						$pb_tmce .= '<button type="button" id="' . esc_attr($editor_id) . '-tmce" class="wp-switch-editor switch-tmce" data-wp-editor-id="' . esc_attr($editor_id) . '">Visual</button>';
						$pb_tmce .= '<button type="button" id="' . esc_attr($editor_id) . '-html" class="wp-switch-editor switch-html" data-wp-editor-id="' . esc_attr($editor_id) . '">Text</button>';
					}
					$pb_tmce .= '</div>'; // wp-editor-tabs
					$pb_tmce .= '</div>'; // wp-editor-tools
				}
				
				// content editor area
				$pb_tmce_content  = '<div id="wp-' . esc_attr($editor_id) . '-editor-container" class="wp-editor-container">';
				$pb_tmce_content .= '<div id="qt_' . esc_attr($editor_id) . '_toolbar" class="quicktags-toolbar"></div>';
				$pb_tmce_content .= '<textarea style="height:300px;" class="' . esc_attr($set['editor_class']) . '" autocomplete="off" cols="40" name="' . esc_attr($set['textarea_name']) . '" id="' . esc_attr($editor_id) . '"></textarea>';
				$pb_tmce_content .= '</div>'; // wp-editor-container
				
				$pb_tmce .= apply_filters('the_editor', $pb_tmce_content);
				$pb_tmce .= '</div>'; // wp-wrap
				
				// remove the fullscreen tmce plugin
				add_filter('tiny_mce_plugins', 'gdlr_core_html_option::tmce_init_plugin', 10, 2);
				
				// action for editor style
				wp_print_styles('editor-buttons');
				_WP_Editors::editor_settings($editor_id, $set);

				$pb_tmce = preg_replace('/[\x00-\x1F\x80-\xFF]/', '', $pb_tmce);
				return $pb_tmce;
			} // tmce_init
			static function tmce_init_plugin($plugins){
				// remove fullscreen option
				if( ($key = array_search('fullscreen', $plugins)) !== false ){
					unset($plugins[$key]);
				}
				return $plugins;
			} // tmce_init_plugin
			
			//////////////////////////////////////////////
			// ajax action
			//////////////////////////////////////////////
			
			static function get_gallery_options(){
				
				if( !check_ajax_referer('gdlr_core_html_option', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( empty($_POST['options']) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('An Error Occurs', 'goodlayers-core'),
						'message'=> esc_html__('No options defined.' ,'goodlayers-core')
					)));
				}else{
					$_POST['options'] = empty($_POST['options'])? array(): gdlr_core_process_post_data($_POST['options']);
					$_POST['value'] = empty($_POST['value'])? array(): gdlr_core_process_post_data($_POST['value']);
					
					$content  = '<div class="gdlr-core-gallery-lb-options" >';
					$content .= '<div class="gdlr-core-gallery-lb-head" >';
					$content .= '<i class="fa fa-save"></i>';
					$content .= '<span class="gdlr-core-head">' . esc_html__('Gallery Image Options', 'goodlayers-core') . '</span>';
					$content .= '<div class="gdlr-core-gallery-lb-head-close" id="gdlr-core-gallery-lb-head-close" ></div>';
					$content .= '</div>'; // gdlr-core-gallery-lb-head
					
					$content .= '<div class="gdlr-core-gallery-lb-options" >';
					foreach( $_POST['options'] as $option_slug => $option_val ){
						$option_val['slug'] = $option_slug;
						if( !empty($_POST['value'][$option_slug]) ){
							$option_val['value'] = $_POST['value'][$option_slug];
						}
						$content .= gdlr_core_html_option::get_element($option_val);	
					}
					$content .= '</div>'; // gdlr-core-gallery-lb-content
					
					$content .= '<div class="gdlr-core-gallery-lb-options-save" id="gdlr-core-gallery-lb-options-save" >';
					$content .= '<i class="fa fa-save"></i>' . esc_html__('Save Options', 'goodlayers-core');
					$content .= '</div>';
					$content .= '</div>'; // gdlr-core-gallery-lb-options
					
				}
				
				die( json_encode(array(
					'status' => 'success',
					'option_content' => $content
				)) ); 
				
			}
			
		} // gdlr_core_html_option
	
	} // class_exists