<?php
	/*	
	*	Goodlayers Item For Wrapper Section
	*/
	
	gdlr_core_page_builder::add_section('template', 'gdlr_core_page_builder_template');
	
	if( !class_exists('gdlr_core_page_builder_template') ){
		class gdlr_core_page_builder_template extends gdlr_core_page_builder_section{

			// use to init the element
			static function init(){
				add_action('wp_ajax_gdlr_core_get_pb_template', 'gdlr_core_page_builder_template::get_template');
			}
			
			// get the section settings
			static function get_settings(){
				return array(	
					'icon' => GDLR_CORE_URL . '/framework/images/page-builder/nav-template.png', 
					'title' => esc_html__('Templates', 'goodlayers-core')
				);
			}
			
			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var = array() ){
	
				$page_builder_var['template']['template']  = '<div class="gdlr-core-page-builder-template" data-template="template" >';
				$page_builder_var['template']['template'] .= '<div class="gdlr-core-page-builder-template-inner" ></div>';
				$page_builder_var['template']['template'] .= '</div>';
				
				return $page_builder_var;
			}					
			
			// get element list for page builder nav bar
			static function get_element_list( $args = array() ){
				
				// search
				echo '<input type="text" placeholder="' . esc_attr('Search Templates', 'goodlayers-core') . '" class="gdlr-core-page-builder-head-content-search" />';
				
				// template type
				echo '<div class="gdlr-core-page-builder-head-content-template-type" >';
				echo '<div class="gdlr-core-template-type-button gdlr-core-active" data-type="page" >' . esc_html__('Pre-Built Pages', 'goodlayers-core') . '</div>';
				echo '<div class="gdlr-core-template-type-button" data-type="block" >' . esc_html__('Pre-Built Blocks', 'goodlayers-core') . '</div>';
				echo '</div>';
				
				// template page
				echo '<div class="gdlr-core-page-builder-head-content-template-container gdlr-core-active" data-type="page" >';

				$pb_page_templates = apply_filters('gdlr_core_page_builder_page_template_list', array());
				foreach( $pb_page_templates as $template_slug => $template_option ){
					echo '<div class="gdlr-core-page-builder-head-content-template-item gdlr-core-pb-list-draggable" data-template="template" '; 
					echo 'data-title="' . esc_attr($template_option['title']) . '" '; 
					echo 'data-type="' . esc_attr($template_option['type']) . '" '; 
					echo 'data-template-slug="' . esc_attr($template_slug) . '" >';
					if( !empty($template_option['thumbnail']) ){
						echo '<img src="' . esc_url($template_option['thumbnail']) . '" alt="" />';
					}
					echo gdlr_core_escape_content($template_option['title']) . '</div>';	
				}
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-content-template-container	
				
				// template block
				echo '<div class="gdlr-core-page-builder-head-content-template-container" data-type="block" >';
				
				$pb_block_templates = apply_filters('gdlr_core_page_builder_block_template_list', array());
				foreach( $pb_block_templates as $template_slug => $template_option ){
					echo '<div class="gdlr-core-page-builder-head-content-template-item gdlr-core-pb-list-draggable" data-template="template" '; 
					echo 'data-title="' . esc_attr($template_option['title']) . '" '; 
					echo 'data-type="' . esc_attr($template_option['type']) . '" '; 
					echo 'data-template-slug="' . esc_attr($template_slug) . '" >';
					if( !empty($template_option['thumbnail']) ){
						echo '<img src="' . esc_url($template_option['thumbnail']) . '" alt="" />';
					}
					echo '</div>';	
				}
				
				echo '<div class="clear"></div>';
				echo '</div>'; // gdlr-core-page-builder-head-content-template-container
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

					$pb_page_template = apply_filters('gdlr_core_page_builder_get_page_template', '', $_POST['slug']);
					
					if( !empty($pb_page_template) ){
						$pb_data = json_decode($pb_page_template, true);
					}else{
						$pb_block_template = apply_filters('gdlr_core_page_builder_get_block_template', '', $_POST['slug']);

						if( !empty($pb_block_template) ){
							$pb_data = json_decode($pb_block_template, true);
						}
					}
					
					$content = gdlr_core_page_builder::get_page_builder_item($pb_data);

					die(json_encode(array(
						'status' => 'success',
						'content' => $content
					))); 
				}
				
			}
			
		} // gdlr_core_page_builder_template
	} // class_exists	