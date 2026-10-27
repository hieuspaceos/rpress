<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('wordpress-editor-content', 'gdlr_core_pb_element_wordpress_editor_content'); 
	
	if( !class_exists('gdlr_core_pb_element_wordpress_editor_content') ){
		class gdlr_core_pb_element_wordpress_editor_content{
			
			// get the element settings
			static function get_settings(){
				return array(
					'title' => esc_html__('Wordpress Editor Content', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
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
?><script id="gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-text-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
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
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$ret  = '<div class="gdlr-core-text-box-item gdlr-core-item-pdlr gdlr-core-item-pdb" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-wordpress-editor-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['font-size'])? '': $settings['font-size']
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}

				if( $preview ){
					$ret .= '<div class="gdlr-core-external-plugin-message">';
					$ret .= esc_html__('This item pulls content from default "wordpress editor" to be shown among the page builder items on the front end of the site.', 'goodlayers-core');
					$ret .= ' ' . esc_html__('You may have to disable the "Show Wordpress Editor Content" at the "Page Option" area out to prevent duplicate content.', 'goodlayers-core');
					$ret .= '</div>';
				}else{
					ob_start();
					the_content();
					$ret .= ob_get_contents();
					ob_end_clean();			
				}

				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_wordpress_editor_content
	} // class_exists	