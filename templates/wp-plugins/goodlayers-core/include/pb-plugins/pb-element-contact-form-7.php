<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('contact-form-7', 'gdlr_core_pb_element_contact_form_7'); 
	
	if( !class_exists('gdlr_core_pb_element_contact_form_7') ){
		class gdlr_core_pb_element_contact_form_7{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-envelope-o',
					'title' => esc_html__('Contact Form 7', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'cf7-id' => array(
								'title' => esc_html__('Choose Contact Form', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_post_list('wpcf7_contact_form')
							),			
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);			
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'cf7-id' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-contact-form-7-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				// display
				if( !function_exists('wpcf7_contact_form_tag_func') ){
					$message = wp_kses(__('Please install and activate the "<a target="_blank" href="https://wordpress.org/plugins/contact-form-7/" >Contact form 7</a>" plugin to show the contact form.', 'goodlayers-core'), 
						array( 'a' => array('target'=>array(), 'href'=>array()) ));
				}else if( empty($settings['cf7-id']) ){
					$message = esc_html__('Please choose the contact form you want to display.', 'goodlayers-core');
				}else if( $preview ){
					$message = '[contact-form-7 id="' . esc_attr($settings['cf7-id']) . '"]';
				}else{
					$ret .= wpcf7_contact_form_tag_func(array('id'=>$settings['cf7-id']), null, 'contact-form-7');
				}
				if( !empty($message) ){
					$ret .= '<div class="gdlr-core-external-plugin-message">' . gdlr_core_escape_content($message) . '</div>';
				}
				
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	