<?php
	/*	
	*	Goodlayers Item For Wrapper Section
	*/
	
	gdlr_core_page_builder::add_section('custom-template', 'gdlr_core_page_builder_custom_template');
	
	if( !class_exists('gdlr_core_page_builder_custom_template') ){
		class gdlr_core_page_builder_custom_template extends gdlr_core_page_builder_section{
			
			private static $pb_custom_templates_slug = 'gdlr-core-pb-custom-template';
			private static $pb_custom_templates = array();

			// use to init the element
			static function init(){
				self::$pb_custom_templates = get_option(self::$pb_custom_templates_slug, array());

				add_action('wp_ajax_gdlr_core_get_pb_custom_template', 'gdlr_core_page_builder_custom_template::get_template');
				add_action('wp_ajax_gdlr_core_save_pb_custom_template', 'gdlr_core_page_builder_custom_template::save_template');
				add_action('wp_ajax_gdlr_core_remove_pb_custom_template', 'gdlr_core_page_builder_custom_template::remove_template');
			}
			
			// get the section settings
			static function get_settings(){
				return array(	
					'icon' => GDLR_CORE_URL . '/framework/images/page-builder/nav-custom-template.png', 
					'title' => esc_html__('Custom Templates', 'goodlayers-core')
				);
			}

			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var = array() ){
	
				// use the same function as template 
				$page_builder_var['template']['custom_template_lb']  = '<div class="gdlr-core-custom-template-lb-content-wrapper">';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-lb-content">';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-lb-head">';
				$page_builder_var['template']['custom_template_lb'] .= '<i class="fa fa-save" ></i>';
				$page_builder_var['template']['custom_template_lb'] .= '<span class="gdlr-core-head" >' . esc_html__('Save Custom Template', 'goodlayers-core') . '</span>';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-lb-head-close" ></div>';
				$page_builder_var['template']['custom_template_lb'] .= '</div>'; // gdlr-core-custom-template-lb-head
				
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-lb-body">';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-name-title" >' . esc_html__('Template Name :', 'goodlayers-core') . '</div>';
				$page_builder_var['template']['custom_template_lb'] .= '<input type="text" class="gdlr-core-custom-template-name" />';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="clear" ></div>';
				$page_builder_var['template']['custom_template_lb'] .= '<div class="gdlr-core-custom-template-save" >' . esc_html__('Save Template', 'goodlayers-core') . '</div>';
				$page_builder_var['template']['custom_template_lb'] .= '</div>'; // gdlr-core-custom-template-lb-body
				$page_builder_var['template']['custom_template_lb'] .= '</div>'; // gdlr-core-custom-template-lb-content
				$page_builder_var['template']['custom_template_lb'] .= '</div>'; // gdlr-core-custom-template-lb-content-wrapper
				
				$page_builder_var['template']['custom_template_no_text_head'] = esc_html__('Undefined Template Name', 'goodlayers-core');
				$page_builder_var['template']['custom_template_no_text_message'] = esc_html__('Please fill in the template name to proceed.', 'goodlayers-core');
				
				return $page_builder_var;
			}		
			
			// get element list for page builder nav bar
			static function get_element_list( $args = array() ){
				
				// search
				echo '<input type="text" placeholder="' . esc_attr('Search Templates', 'goodlayers-core') . '" class="gdlr-core-page-builder-head-content-search" />';
				
				echo '<div class="gdlr-core-page-builder-head-content-custom-template-container clearfix" >';
				
				if( !empty(self::$pb_custom_templates) ){
					foreach( self::$pb_custom_templates as $template_slug => $template_option ){
						echo self::get_element_item($template_slug, $template_option);
					}
				}
				
				echo '</div>'; // gdlr-core-page-builder-head-content-template-container	
				
			}	
			static function get_element_item($slug, $option){
				$ret  = '<div class="gdlr-core-page-builder-head-content-custom-template-item gdlr-core-pb-list-draggable" data-template="custom-template" '; 
				$ret .= 'data-title="' . esc_attr($option['title']) . '" '; 
				$ret .= 'data-type="' . esc_attr($option['type']) . '" '; 
				$ret .= 'data-template-slug="' . esc_attr($slug) . '" >';
				$ret .= '<span class="gdlr-core-page-builder-head-content-custom-template-title" >' . $option['title'] . '</span>';
				$ret .= '<div class="gdlr-core-page-builder-head-content-custom-template-remove" >';
				$ret .= '<i class="fa fa-remove" ></i>';
				$ret .= '</div>';
				$ret .= '</div>';
				
				return $ret;
			}
			
			// get template for page builder
			static function get_template( $options = array(), $callback = '' ){
				
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['slug']) ){
					
					$pb_data = self::$pb_custom_templates[$_POST['slug']]['value'];
					$content = gdlr_core_page_builder::get_page_builder_item($pb_data);
					
					die(json_encode(array(
						'status' => 'success',
						'content' => $content
					))); 
				}
				
			}
			
			// save page builder template
			static function add_template( $templates ){

				$count = 0;
				if( !empty($templates) && is_array($templates) ){
					foreach( $templates as $template ){
						while( !empty(self::$pb_custom_templates['gdlr-core-custom-template-' . $count]) ){ $count++; }
						self::$pb_custom_templates['gdlr-core-custom-template-' . $count] = $template;
					}
					update_option(self::$pb_custom_templates_slug, self::$pb_custom_templates, false);
				}

			}
			static function save_template(){
				
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['value']) && !empty($_POST['title']) && !empty($_POST['type']) ){
					
					// create the custom template slug
					$count = 0;
					while( !empty(self::$pb_custom_templates['gdlr-core-custom-template-' . $count]) ){ $count++; }
					$custom_template_slug = 'gdlr-core-custom-template-' . $count;
					
					self::$pb_custom_templates[$custom_template_slug] = array(
						'title' => gdlr_core_process_post_data($_POST['title']),
						'type' => gdlr_core_process_post_data($_POST['type']),
						'value' => gdlr_core_process_post_data($_POST['value'])
					);
					self::$pb_custom_templates[$custom_template_slug] = gdlr_core_sanitize_input(self::$pb_custom_templates[$custom_template_slug]);
					update_option(self::$pb_custom_templates_slug, self::$pb_custom_templates, false);

					die(json_encode(array(
						'status' => 'success',
						'head' => esc_html__('Template Added', 'goodlayers-core'),
						'message' => '',
						'nav_item' => self::get_element_item($custom_template_slug, self::$pb_custom_templates[$custom_template_slug])
					))); 
				}

				$count = is_array($_POST['value'])? count($_POST['value'], 1): 0;
				die(json_encode(array(
					'status' => 'failed',
					'head' => esc_html__('Incomplete Data', 'goodlayers-core'),
					'message' => 'value:: ' . $count
				)));
				
			}	

			// remove page builder template
			static function remove_template(){
				
				if( !check_ajax_referer('gdlr_core_page_builder', 'security', false) ){
					die(json_encode(array(
						'status' => 'failed',
						'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
						'message'=> esc_html__('Unable to remove the template. Please refresh the page and try again.' ,'goodlayers-core')
					)));
				}
				
				if( !empty($_POST['slug']) ){
					
					unset(self::$pb_custom_templates[$_POST['slug']]);
					update_option(self::$pb_custom_templates_slug, self::$pb_custom_templates, false);

					die(json_encode(array('status' => 'success'))); 
				}
				
			}				
			
		} // gdlr_core_page_builder_custom_template
	} // class_exists	