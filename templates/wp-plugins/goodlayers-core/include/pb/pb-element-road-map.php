<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('roadmap', 'gdlr_core_pb_element_roadmap'); 
	
	if( !class_exists('gdlr_core_pb_element_roadmap') ){
		class gdlr_core_pb_element_roadmap{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Road Map', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'tabs' => array(
								'title' => esc_html__('Add New Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'content-title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
									),
								),
								'default' => array(
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
									array(
										'title' => esc_html__('Tab Title', 'goodlayers-core'),
										'content' => esc_html__('Tab content area', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					)
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-roadmap-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-roadmap-<?php echo esc_attr($id); ?>').parent().gdlr_core_tab();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;

				// default variable
				if( empty($settings) ){
					$settings = array(
						'tabs' => array(
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
							array(
								'title' => esc_html__('Tab Title', 'goodlayers-core'),
								'content' => esc_html__('Tab content area', 'goodlayers-core'),
							),
						),
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
					
				// start printing item
				$ret  = '<div class="gdlr-core-roadmap-item gdlr-core-js gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['tabs']) ){
					$count = 0; $active = 1;

					$ret .= '<div class="gdlr-core-roadmap-item-head-wrap" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-roadmap-item-head clearfix ' . ($count == $active? 'gdlr-core-active': '') . '" data-tab-id="' . esc_attr($count) . '" >';
						$ret .= '<div class="gdlr-core-roadmap-item-head-count" >' . $count . '</div>';
						$ret .= '<div class="gdlr-core-roadmap-item-head-divider" ></div>';
						$ret .= '<div class="gdlr-core-roadmap-item-head-content" >';
						if( !empty($tab['title']) ){
							$ret .= '<h3 class="gdlr-core-roadmap-item-head-title" >' . gdlr_core_text_filter($tab['title']) . '</h3>';
						}
						if( !empty($tab['caption']) ){
							$ret .= '<div class="gdlr-core-roadmap-item-head-caption" >' . gdlr_core_text_filter($tab['caption']) . '</div>';
						}
						$ret .= '</div>'; // gdlr-core-roadmap-item-head-content
						$ret .= '</div>'; // gdlr-core-roadmap-item-head
					}
					$ret .= '</div>'; // gdlr-core-roadmap-item-head-wrap

					$count = 0;
					$ret .= '<div class="gdlr-core-roadmap-item-content-wrap" >';
					foreach( $settings['tabs'] as $tab ){ $count++;
						$ret .= '<div class="gdlr-core-roadmap-item-content-area ' . ($count == $active? 'gdlr-core-active': '') . '" data-tab-id="' . esc_attr($count) . '" >';
						if( !empty($tab['content-caption']) ){
							$ret .= '<div class="gdlr-core-roadmap-item-content-caption" >' . gdlr_core_text_filter($tab['content-caption']) . '</div>';
						}
						if( !empty($tab['content-title']) ){
							$ret .= '<h3 class="gdlr-core-roadmap-item-content-title" >' . gdlr_core_text_filter($tab['content-title']) . '</h3>';
						}
						if( !empty($tab['content']) ){
							$ret .= '<div class="gdlr-core-roadmap-item-content" >' . gdlr_core_text_filter($tab['content']) . '</div>';
						} 
						$ret .= '</div>';
					}
					$ret .= '</div>'; // gdlr-core-roadmap-item-content-wrap
				}
				$ret .= '</div>'; // gdlr-core-roadmap-item

				return $ret;
			}			
			
		} // gdlr_core_pb_element_tab
	} // class_exists	