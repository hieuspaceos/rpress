<?php
	/*	
	*	Goodlayers Item For Wrapper Section
	*/
	gdlr_core_page_builder::add_section('sync-template', 'gdlr_core_page_builder_sync_template');
	
	if( !class_exists('gdlr_core_page_builder_sync_template') ){
		class gdlr_core_page_builder_sync_template extends gdlr_core_page_builder_section{
			
			private static $pb_sync_templates_slug = 'gdlr-core-pb-sync-template';
			private static $pb_sync_templates = array();

			// use to init the element
			static function init(){
				self::$pb_sync_templates = get_option(self::$pb_sync_templates_slug, array());

				add_action('wp_ajax_gdlr_core_get_pb_sync_template', 'gdlr_core_page_builder_sync_template::get_template');
				add_action('wp_ajax_gdlr_core_save_pb_sync_template', 'gdlr_core_page_builder_sync_template::save_template');
				add_action('wp_ajax_gdlr_core_remove_pb_sync_template', 'gdlr_core_page_builder_sync_template::remove_template');
			}
			
			// get the section settings
			static function get_settings(){
				return array(	
					'icon' => GDLR_CORE_URL . '/framework/images/page-builder/nav-sync-template.png', 
					'title' => esc_html__('Sync Templates', 'goodlayers-core')
				);
			}

			// assign the page builder variable
			static function set_page_builder_var( $page_builder_var = array() ){

				$page_builder_var['sync_template']['sync_template_remove_head'] = esc_html__('You are removing synced wrapper!', 'goodlayers-core');
				$page_builder_var['sync_template']['sync_template_remove_message'] = esc_html__('Please note that by doing this, all synced wrapper will be unsynced and they will be independent. Please make sure before doing this. This process can\'t be undone.', 'goodlayers-core');

				return $page_builder_var;
			}		
			
			// get element list for page builder nav bar
			static function get_element_list( $args = array() ){
				
				// search
				echo '<input type="text" placeholder="' . esc_attr('Search Templates', 'goodlayers-core') . '" class="gdlr-core-page-builder-head-content-search" />';
				echo '<div class="gdlr-core-page-builder-head-content-sync-template-container clearfix" >';
				if( !empty(self::$pb_sync_templates) ){
					foreach( self::$pb_sync_templates as $template_slug => $template_option ){
						echo self::get_element_item($template_slug, $template_option);
					}
				}
				echo '</div>'; // gdlr-core-page-builder-head-content-template-container	
				
			}	
			static function get_element_item($slug, $option){
				$ret  = '<div class="gdlr-core-page-builder-head-content-sync-template-item gdlr-core-pb-list-draggable" data-template="sync-template" '; 
				$ret .= 'data-title="' . esc_attr($option['title']) . '" '; 
				$ret .= 'data-type="' . esc_attr($option['type']) . '" '; 
				$ret .= 'data-template-slug="' . esc_attr($slug) . '" >';
				$ret .= '<span class="gdlr-core-page-builder-head-content-sync-template-title" >' . $option['title'] . '</span>';
				$ret .= '<div class="gdlr-core-page-builder-head-content-sync-template-remove" >';
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
					
					$pb_data = self::$pb_sync_templates[$_POST['slug']];
					$content = gdlr_core_page_builder::get_page_builder_item($pb_data);
					
					die(json_encode(array(
						'status' => 'success',
						'content' => $content
					))); 
				}
				
			}
			static function get_sync_template( $slug ){

				if( !empty(self::$pb_sync_templates[$slug]) ){
					return self::$pb_sync_templates[$slug]['value'];
				}

				return array();
			}
			
			// save page builder template
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
					$sync_template_slug = gdlr_core_process_post_data($_POST['title']);
					$sync_template_val = gdlr_core_process_post_data($_POST['value']);
					$sync_template_val[0]['sync-template'] = $sync_template_slug;

					$new_nav_item = false;
					if( empty(self::$pb_sync_templates[$sync_template_slug]) ){
						$new_nav_item = true;
					}

					self::$pb_sync_templates[$sync_template_slug] = array(
						'title' => $sync_template_slug,
						'type' => gdlr_core_process_post_data($_POST['type']),
						'value' => $sync_template_val[0]
					);
					update_option(self::$pb_sync_templates_slug, self::$pb_sync_templates, false);

					die(json_encode(array(
						'status' => 'success',
						'head' => esc_html__('Template Added', 'goodlayers-core'),
						'message' => '',
						'sync_template_slug' => $sync_template_slug,
						'nav_item' => $new_nav_item? self::get_element_item($sync_template_slug, self::$pb_sync_templates[$sync_template_slug]): ''
					))); 
				}

				die(json_encode(array(
					'status' => 'failed',
					'head' => esc_html__('Incomplete Data', 'goodlayers-core'),
					'message' => 'value:: ' . count($_POST['value'], 1)
				)));
				
			}
			static function update_sync_template( $pb_value ){

				foreach( $pb_value as $wrapper ){
					if( !empty($wrapper['sync-template']) ){
						$sync_slug = $wrapper['sync-template'];
						if( !empty(self::$pb_sync_templates[$sync_slug]) ){
							self::$pb_sync_templates[$sync_slug] = array(
								'title' => $sync_slug,
								'type' => $wrapper['template'],
								'value' => $wrapper
							);
						}
					}
				} 

				update_option(self::$pb_sync_templates_slug, self::$pb_sync_templates, false);
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
					
					unset(self::$pb_sync_templates[$_POST['slug']]);
					update_option(self::$pb_sync_templates_slug, self::$pb_sync_templates, false);

					die(json_encode(array('status' => 'success'))); 
				}
				
			}				
			
		} // gdlr_core_page_builder_custom_template
	} // class_exists	