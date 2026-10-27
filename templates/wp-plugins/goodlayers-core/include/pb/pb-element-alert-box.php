<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('alert-box', 'gdlr_core_pb_element_alert_box'); 
	
	if( !class_exists('gdlr_core_pb_element_alert_box') ){
		class gdlr_core_pb_element_alert_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-warning',
					'title' => esc_html__('Alert Box', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'default' => 'fa fa-warning',
								'allow-none' => true,
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Alert Box Item Title', 'goodlayers-core'),
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Alert box item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),						
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'removable' => array(
								'title' => esc_html__('Removable ( Close Button )', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'left'
							),
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '3px'
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => ''
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
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-alert-box-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-alert-box-<?php echo esc_attr($id); ?>').parent().gdlr_core_content_script();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'icon' => 'fa fa-warning',
						'title' => esc_html__('Alert Box Item Title', 'goodlayers-core'),
						'content' => esc_html__('Alert box item sample content', 'goodlayers-core'),
						
						'text-align' => 'left', 'icon-color' => '', 'title-color' => '', 'content-color' => '', 'background-color' => '', 'border-width' => '2px', 'border-color' => '',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['icon'])? ' gdlr-core-no-icon': '';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-alert-box-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom' => $settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= '<div class="gdlr-core-alert-box-item-inner gdlr-core-skin-e-background gdlr-core-skin-border" ' . gdlr_core_esc_style(array(
					'background-color' => empty($settings['background-color'])? '': $settings['background-color'],
					'border-color' => empty($settings['border-color'])? '': $settings['border-color'],
					'border-width' => empty($settings['border-width'])? '': $settings['border-width'],
				)) . '>';
				if( empty($settings['removable']) || $settings['removable'] == 'enable' ){
					$ret .= '<div class="gdlr-core-alert-box-remove gdlr-core-js" ><i class="fa fa-remove" ></i></div>';
				} 
				$ret .= '<div class="gdlr-core-alert-box-item-head">';
				if( !empty($settings['icon']) ){
					$ret .= '<div class="gdlr-core-alert-box-item-icon" ><i class="' . esc_attr($settings['icon']) . '" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['icon-color'])? '': $settings['icon-color']
					)) . ' ></i></div>';
				}
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-alert-box-item-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['title-color'])? '': $settings['title-color']
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				$ret .= '</div>'; // gdlr-core-alert-box-item-head
				
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-alert-box-item-content gdlr-core-skin-e-content" ' . gdlr_core_esc_style(array(
						'color' => empty($settings['content-color'])? '': $settings['content-color']
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				$ret .= '</div>'; // gdlr-core-alert-box-item-inner
				$ret .= '</div>'; // gdlr-core-alert-box-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	